#!/usr/bin/env python3
"""Bounded filesystem metadata accounting for qualification profiles; no content reads."""
import argparse
import json
import os
from pathlib import Path
import stat
import time


def measure_profile_storage(home, max_entries=100000):
    home = Path(home).resolve(strict=True)
    if not home.is_dir() or not 1 <= max_entries <= 1000000:
        raise ValueError('Select a directory and an explicit bounded entry limit')
    started = time.time_ns()
    groups, seen, errors = {}, set(), []
    entries = links = duplicate_inodes = 0
    bounded = False
    # Directory-relative open with O_NOFOLLOW prevents a concurrent symlink
    # replacement from redirecting traversal outside the selected profile.
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    def scan(directory_fd, parts):
        nonlocal entries, links, duplicate_inodes, bounded
        with os.scandir(directory_fd) as children:
            for entry in children:
                if entries >= max_entries:
                    bounded = True
                    return
                entries += 1
                try:
                    info = entry.stat(follow_symlinks=False)
                    if stat.S_ISLNK(info.st_mode):
                        links += 1
                        continue
                    current = (*parts, entry.name)
                    if stat.S_ISDIR(info.st_mode):
                        if len(current) >= 64:
                            errors.append({'category':'depth_limit', 'errno':None})
                            continue
                        child_fd = os.open(entry.name, flags, dir_fd=directory_fd)
                        try:
                            opened = os.fstat(child_fd)
                            if (opened.st_dev, opened.st_ino) != (info.st_dev, info.st_ino):
                                errors.append({'category':'directory_changed','errno':None})
                            else:
                                scan(child_fd, current)
                        finally:
                            os.close(child_fd)
                        if bounded:
                            return
                        continue
                    if not stat.S_ISREG(info.st_mode):
                        continue
                    inode = (info.st_dev, info.st_ino)
                    if inode in seen:
                        duplicate_inodes += 1
                        continue
                    seen.add(inode)
                    group = '/'.join(current[:2]) if len(current) > 2 else current[0] if len(current) > 1 else 'profile-root'
                    values = groups.setdefault(group, {'files':0,'logical_bytes':0,'allocated_bytes':0})
                    values['files'] += 1
                    values['logical_bytes'] += info.st_size
                    if not hasattr(info, 'st_blocks'):
                        values['allocated_bytes'] = None
                    elif values['allocated_bytes'] is not None:
                        values['allocated_bytes'] += info.st_blocks * 512
                except OSError as error:
                    errors.append({'category':'entry_unavailable','errno':error.errno})
    root_fd = os.open(home, flags)
    try:
        scan(root_fd, ())
    except OSError as error:
        errors.append({'category':'directory_unavailable','errno':error.errno})
    finally:
        os.close(root_fd)
    allocated = [value['allocated_bytes'] for value in groups.values()]
    return {
        'schema':'yai.qualification_profile_storage.v1',
        'scope':'whole_profile_not_per_case',
        'started_at_unix_ns':started,
        'completed_at_unix_ns':time.time_ns(),
        'posture':'partial' if errors or bounded else 'observed',
        'entry_limit':max_entries,
        'entry_limit_reached':bounded,
        'symlinks_not_followed':links,
        'duplicate_inodes_not_recounted':duplicate_inodes,
        'containers':dict(sorted(groups.items())),
        'unique_files':len(seen),
        'logical_bytes':sum(value['logical_bytes'] for value in groups.values()),
        'allocated_bytes':sum(allocated) if all(value is not None for value in allocated) else None,
        'errors':errors,
        'limitations':[
            'Non-atomic filesystem metadata observation; concurrent changes may occur',
            'Regular-file bytes only; directory metadata and reflink sharing are not attributed',
            'Container bytes do not distinguish canonical, retained and derived records inside a database',
            'No per-Case attribution or model-storage observation',
            'Hard-linked inodes counted once; reflink sharing is not measured',
            'No files read, changed, compacted or deleted',
        ],
    }


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--home',type=Path,required=True)
    parser.add_argument('--max-entries',type=int,default=100000)
    args=parser.parse_args()
    result=measure_profile_storage(args.home,args.max_entries)
    print(json.dumps(result,indent=2))
    raise SystemExit(0 if result['posture']=='observed' else 2)


if __name__=='__main__':
    main()
