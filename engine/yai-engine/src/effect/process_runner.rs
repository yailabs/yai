//! Physical runner inside the controlled-effect owner. This is not admission.
//! Initial profile: read-only workspace, system runtime reads, no network or
//! descendants. It cannot be used as a general shell/build environment.
//! The application carrier must acquire and revalidate its exact effect fence.

use super::access::{LocalAccessBinding, ResourceAddress, MAX_ACCESS_BYTES};
use super::{digest_bytes, open_beneath, open_verified_filesystem_root};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub const PROCESS_PROFILE: &str = super::access::PROCESS_EFFECT_BACKEND;

#[derive(Debug, Serialize)]
pub struct BoundedProcessResult {
    pub profile: &'static str,
    pub executable_digest: String,
    pub configuration_digest: String,
    pub argv: Vec<String>,
    pub working_directory: String,
    pub environment_keys: Vec<String>,
    pub timeout_ms: u64,
    pub elapsed_ms: u128,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub timed_out: bool,
    pub output_limit_exceeded: bool,
    pub stdout: String,
    pub stderr: String,
    pub stdout_digest: String,
    pub stderr_digest: String,
    pub lossy_utf8: bool,
}

/// Descriptor-pinned executable and cwd plus an already constructed kernel
/// ruleset. Preparation performs no child execution and does not grant policy.
pub struct BoundedProcess {
    executable: File,
    directory: File,
    ruleset: File,
    binding: LocalAccessBinding,
    runner: super::access::ProcessRunner,
    output_limit: usize,
}

impl BoundedProcess {
    pub fn prepare(
        binding: &LocalAccessBinding,
        name: &str,
        output_limit: usize,
    ) -> Result<Self, String> {
        binding.validate()?;
        if output_limit == 0 || output_limit > MAX_ACCESS_BYTES {
            return Err("process_output_bound_invalid".into());
        }
        if unsafe { libc::geteuid() } == 0 {
            return Err("process_runner_requires_unprivileged_host".into());
        }
        let ResourceAddress::ProcessRunner { root, runners } = &binding.address else {
            return Err("process_runner_resource_required".into());
        };
        let runner = runners.get(name).ok_or("process_runner_not_bound")?.clone();
        let root = open_verified_filesystem_root(root)?;
        let directory = open_beneath(
            &root,
            Path::new(&runner.working_directory),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
            0,
        )
        .map_err(|e| format!("process_cwd_refused:{e}"))?;
        let mut executable = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(&runner.executable)
            .map_err(|e| format!("process_executable_open:{e}"))?;
        let before = executable.metadata().map_err(|e| e.to_string())?;
        if !before.is_file()
            || before.len() > 128 * 1024 * 1024
            || before.mode() & 0o111 == 0
            || before.uid() != 0
            || before.mode() & 0o022 != 0
        {
            return Err("process_executable_not_bounded_regular_binary".into());
        }
        let mut bytes = Vec::new();
        (&mut executable)
            .take(128 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        let after = executable.metadata().map_err(|e| e.to_string())?;
        if !bytes.starts_with(b"\x7fELF")
            || digest_bytes(&bytes) != runner.executable_digest
            || before.ctime() != after.ctime()
            || before.ctime_nsec() != after.ctime_nsec()
            || before.len() != after.len()
        {
            return Err("process_executable_integrity_mismatch".into());
        }
        let ruleset = readonly_ruleset(&root, &executable)?;
        Ok(Self {
            executable,
            directory,
            ruleset,
            binding: binding.clone(),
            runner,
            output_limit,
        })
    }

    /// Sole physical spawn site. A caller must run this only after canonical
    /// PREPARE and current fence validation. No automatic retry is implemented.
    pub fn execute(
        self,
        validate_fence: impl FnOnce() -> Result<(), String>,
    ) -> Result<BoundedProcessResult, String> {
        let filter = syscall_filter();
        let program_fd = self.executable.as_raw_fd();
        let cwd_fd = self.directory.as_raw_fd();
        let ruleset_fd = self.ruleset.as_raw_fd();
        let parent_pid = std::process::id() as libc::pid_t;
        let mut command = Command::new(format!("/proc/self/fd/{program_fd}"));
        command
            .arg0(&self.runner.executable)
            .args(&self.runner.argv)
            .env_clear()
            .envs(&self.runner.environment)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let cpu_seconds = self.runner.timeout_ms.div_ceil(1000).saturating_add(1);
        // Only async-signal-safe syscalls in the post-fork child. No allocation,
        // logging, store access or locks are allowed in this closure.
        unsafe {
            command.pre_exec(move || {
                let check = |result: libc::c_long| {
                    if result == -1 {
                        Err(io::Error::last_os_error())
                    } else {
                        Ok(())
                    }
                };
                check(libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) as _)?;
                check(libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) as _)?;
                if libc::getppid() != parent_pid {
                    return Err(io::Error::from_raw_os_error(libc::ECHILD));
                }
                check(libc::fchdir(cwd_fd) as _)?;
                for (resource, limit) in [
                    (libc::RLIMIT_CORE, 0),
                    (libc::RLIMIT_FSIZE, 0),
                    (libc::RLIMIT_NOFILE, 64),
                    (libc::RLIMIT_AS, 512 * 1024 * 1024),
                    (libc::RLIMIT_CPU, cpu_seconds),
                ] {
                    check(libc::setrlimit(
                        resource,
                        &libc::rlimit {
                            rlim_cur: limit,
                            rlim_max: limit,
                        },
                    ) as _)?;
                }
                check(libc::syscall(
                    libc::SYS_landlock_restrict_self,
                    ruleset_fd,
                    0,
                ))?;
                let program = libc::sock_fprog {
                    len: filter.len() as u16,
                    filter: filter.as_ptr() as *mut _,
                };
                check(libc::prctl(libc::PR_SET_SECCOMP, 2, &program) as _)?;
                Ok(())
            });
        }
        validate_fence()?;
        let start = Instant::now();
        let child = command
            .spawn()
            .map_err(|e| format!("process_not_started:{e}"))?;
        let mut child = KillOnDrop(child);
        let mut stdout = child
            .0
            .stdout
            .take()
            .ok_or("process_stdout_unavailable_after_spawn")?;
        let mut stderr = child
            .0
            .stderr
            .take()
            .ok_or("process_stderr_unavailable_after_spawn")?;
        for fd in [stdout.as_raw_fd(), stderr.as_raw_fd()] {
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
            {
                return Err("process_pipe_setup_indeterminate_after_spawn".into());
            }
        }
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let (mut out_done, mut err_done) = (false, false);
        let mut timed_out = false;
        let mut exceeded = false;
        let status = loop {
            out_done |= drain(&mut stdout, &mut out, self.output_limit)?;
            err_done |= drain(&mut stderr, &mut err, self.output_limit)?;
            exceeded |= out.len() > self.output_limit || err.len() > self.output_limit;
            timed_out |= start.elapsed() >= Duration::from_millis(self.runner.timeout_ms);
            if timed_out || exceeded {
                let _ = child.0.kill();
                break child
                    .0
                    .wait()
                    .map_err(|e| format!("process_wait_indeterminate:{e}"))?;
            }
            if let Some(status) = child
                .0
                .try_wait()
                .map_err(|e| format!("process_wait_indeterminate:{e}"))?
            {
                if out_done && err_done {
                    break status;
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        out.truncate(self.output_limit);
        err.truncate(self.output_limit);
        let lossy_utf8 = std::str::from_utf8(&out).is_err() || std::str::from_utf8(&err).is_err();
        Ok(BoundedProcessResult {
            profile: PROCESS_PROFILE,
            executable_digest: self.runner.executable_digest.clone(),
            configuration_digest: self.binding.digest(),
            argv: self.runner.argv.clone(),
            working_directory: self.runner.working_directory.clone(),
            environment_keys: self.runner.environment.keys().cloned().collect(),
            timeout_ms: self.runner.timeout_ms,
            elapsed_ms: start.elapsed().as_millis(),
            exit_code: status.code(),
            signal: status.signal(),
            timed_out,
            output_limit_exceeded: exceeded,
            stdout: String::from_utf8_lossy(&out).into_owned(),
            stderr: String::from_utf8_lossy(&err).into_owned(),
            stdout_digest: digest_bytes(&out),
            stderr_digest: digest_bytes(&err),
            lossy_utf8,
        })
    }
}

struct KillOnDrop(Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn drain(source: &mut impl Read, bytes: &mut Vec<u8>, limit: usize) -> Result<bool, String> {
    let mut buffer = [0u8; 4096];
    while bytes.len() <= limit {
        let take = buffer.len().min(limit + 1 - bytes.len());
        match source.read(&mut buffer[..take]) {
            Ok(0) => return Ok(true),
            Ok(n) => bytes.extend_from_slice(&buffer[..n]),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("process_output_indeterminate_after_spawn:{e}")),
        }
    }
    Ok(false)
}

fn readonly_ruleset(workspace: &File, executable: &File) -> Result<File, String> {
    let abi = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            std::ptr::null::<u8>(),
            0,
            1,
        )
    };
    // ABI 5 admits device-ioctl restrictions. ABI 6 also isolates signal/Unix
    // scopes. Refuse unavailable confinement; never silently run unconfined.
    if abi < 6 {
        return Err(format!(
            "process_confinement_unavailable:landlock_abi={abi}:required=6"
        ));
    }
    #[repr(C)]
    struct Ruleset {
        fs: u64,
        net: u64,
        scoped: u64,
    }
    #[repr(C, packed)]
    struct Rule {
        allowed: u64,
        parent_fd: i32,
    }
    let attributes = Ruleset {
        fs: (1 << 16) - 1,
        net: 3,
        scoped: 3,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            &attributes,
            std::mem::size_of::<Ruleset>(),
            0,
        )
    };
    if fd < 0 {
        return Err(format!(
            "process_confinement_ruleset:{}",
            io::Error::last_os_error()
        ));
    }
    let ruleset = unsafe { File::from_raw_fd(fd as i32) };
    let add = |file: &File, allowed| {
        let rule = Rule {
            allowed,
            parent_fd: file.as_raw_fd(),
        };
        if unsafe { libc::syscall(libc::SYS_landlock_add_rule, fd, 1, &rule, 0) } < 0 {
            Err(format!(
                "process_confinement_rule:{}",
                io::Error::last_os_error()
            ))
        } else {
            Ok(())
        }
    };
    add(workspace, (1 << 2) | (1 << 3))?;
    add(executable, (1 << 0) | (1 << 2))?;
    // Fixed read-only runtime support, not application data or another Case.
    // The executable must be a real ELF; its runtime dependencies are OS files.
    for path in ["/usr", "/lib", "/lib64", "/etc/ld.so.cache"] {
        match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_PATH | libc::O_CLOEXEC)
            .open(path)
        {
            Ok(file) => add(
                &file,
                if file.metadata().map_err(|e| e.to_string())?.is_dir() {
                    1 | 4 | 8
                } else {
                    4
                },
            )?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("process_runtime_root:{e}")),
        }
    }
    Ok(ruleset)
}

/// Allowlist for the bounded Linux ELF/Python test-runner profile. Unknown
/// syscalls, networking, process/thread creation, signalling, writes through
/// unmediated metadata interfaces and io_uring are refused. Filesystem reads
/// and execution remain additionally governed by the Landlock hierarchy.
fn syscall_filter() -> Vec<libc::sock_filter> {
    let instruction = |code, jt, jf, k| libc::sock_filter { code, jt, jf, k };
    let mut filters = vec![
        instruction(0x20, 0, 0, 4),
        instruction(0x15, 1, 0, 0xc000003e),
        instruction(0x06, 0, 0, 0x80000000),
        instruction(0x20, 0, 0, 0),
    ];
    // glibc reads its own limits using prlimit64. Neither another PID nor
    // changing even the child's admitted hard limits is part of this profile.
    // seccomp_data.args starts at byte 16; check both halves of the 64-bit
    // PID and new_limit pointer before allowing this special syscall.
    filters.push(instruction(0x15, 0, 13, libc::SYS_prlimit64 as u32));
    for offset in [16, 20, 32, 36] {
        filters.push(instruction(0x20, 0, 0, offset));
        filters.push(instruction(0x15, 1, 0, 0));
        filters.push(instruction(0x06, 0, 0, 0x00050000 | libc::EPERM as u32));
    }
    filters.push(instruction(0x06, 0, 0, 0x7fff0000));
    // Only read-only terminal/pipe queries and process-local close-on-exec.
    // glibc/Python use TCGETS2 and FIOCLEX. Landlock filesystem permissions
    // alone are not a general filter for metadata-changing file ioctls.
    filters.push(instruction(0x15, 0, 7, libc::SYS_ioctl as u32));
    filters.push(instruction(0x20, 0, 0, 24));
    filters.push(instruction(0x15, 4, 0, libc::TCGETS as u32));
    filters.push(instruction(0x15, 3, 0, libc::FIONREAD as u32));
    filters.push(instruction(0x15, 2, 0, libc::TCGETS2 as u32));
    filters.push(instruction(0x15, 1, 0, libc::FIOCLEX as u32));
    filters.push(instruction(0x06, 0, 0, 0x00050000 | libc::EPERM as u32));
    filters.push(instruction(0x06, 0, 0, 0x7fff0000));
    for syscall in [
        libc::SYS_read,
        libc::SYS_write,
        libc::SYS_close,
        libc::SYS_fstat,
        libc::SYS_newfstatat,
        libc::SYS_stat,
        libc::SYS_lstat,
        libc::SYS_lseek,
        libc::SYS_mmap,
        libc::SYS_mprotect,
        libc::SYS_munmap,
        libc::SYS_brk,
        libc::SYS_rt_sigaction,
        libc::SYS_rt_sigprocmask,
        libc::SYS_rt_sigreturn,
        libc::SYS_pread64,
        libc::SYS_readv,
        libc::SYS_writev,
        libc::SYS_access,
        libc::SYS_open,
        libc::SYS_openat,
        libc::SYS_getdents64,
        libc::SYS_getcwd,
        libc::SYS_fcntl,
        libc::SYS_dup,
        libc::SYS_dup2,
        libc::SYS_dup3,
        libc::SYS_execve,
        libc::SYS_execveat,
        libc::SYS_exit_group,
        libc::SYS_exit,
        libc::SYS_set_tid_address,
        libc::SYS_set_robust_list,
        libc::SYS_rseq,
        libc::SYS_arch_prctl,
        libc::SYS_uname,
        libc::SYS_getpid,
        libc::SYS_getppid,
        libc::SYS_getuid,
        libc::SYS_geteuid,
        libc::SYS_getgid,
        libc::SYS_getegid,
        libc::SYS_clock_gettime,
        libc::SYS_clock_nanosleep,
        libc::SYS_nanosleep,
        libc::SYS_getrandom,
        libc::SYS_sched_getaffinity,
        libc::SYS_futex,
        libc::SYS_statfs,
        libc::SYS_fstatfs,
        libc::SYS_sysinfo,
        libc::SYS_readlink,
        libc::SYS_readlinkat,
        libc::SYS_poll,
        libc::SYS_ppoll,
        libc::SYS_select,
        libc::SYS_pselect6,
        libc::SYS_madvise,
        libc::SYS_getrlimit,
        libc::SYS_getrusage,
        libc::SYS_restart_syscall,
    ] {
        filters.push(instruction(0x15, 0, 1, syscall as u32));
        filters.push(instruction(0x06, 0, 0, 0x7fff0000));
    }
    filters.push(instruction(0x06, 0, 0, 0x00050000 | libc::EPERM as u32));
    filters
}

#[cfg(test)]
mod tests {
    use super::super::access::{ProcessRunner, LOCAL_ACCESS_BINDING_SCHEMA};
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn bounded_runner_real_process_confinement_exit_timeout_and_output() {
        let root = std::env::temp_dir().join(format!(
            "yai-process-component-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("project")).unwrap();
        let outside = root.with_extension("outside");
        std::fs::write(&outside, b"not a Case resource").unwrap();
        let executable = std::fs::canonicalize("/usr/bin/python3").unwrap();
        let digest = digest_bytes(&std::fs::read(&executable).unwrap());
        let runner = ProcessRunner {
            executable: executable.to_str().unwrap().into(),
            executable_digest: digest,
            argv: vec!["-B".into(), "-S".into(), "test.py".into()],
            working_directory: "project".into(),
            environment: BTreeMap::new(),
            timeout_ms: 2_000,
        };
        let mut binding = LocalAccessBinding {
            schema: LOCAL_ACCESS_BINDING_SCHEMA.into(),
            case_id: "case:process-component".into(),
            attachment_id: "resource:runner".into(),
            address: ResourceAddress::ProcessRunner {
                root: super::super::LocalFilesystemBinding::new(
                    "case:process-component",
                    "resource:runner",
                    &root,
                )
                .unwrap(),
                runners: BTreeMap::from([("test".into(), runner)]),
            },
        };
        let script = format!("import os, socket, resource, fcntl\nassert resource.prlimit(0,resource.RLIMIT_NOFILE) == (64,64)\nfor action in [lambda:open({outside:?}).read(), lambda:open('created','w'), lambda:socket.socket(), lambda:os.fork(), lambda:os.chmod({outside:?},0o777), lambda:resource.prlimit(os.getppid(),resource.RLIMIT_NOFILE), lambda:resource.prlimit(0,resource.RLIMIT_NOFILE,(64,64)), lambda:fcntl.ioctl(0,0x5412,b'x')]:\n try:\n  action()\n  raise AssertionError('ambient access')\n except PermissionError:\n  print('denied')\nprint('bounded oracle')\nraise SystemExit(7)\n", outside=outside.to_str().unwrap());
        std::fs::write(root.join("project/test.py"), script).unwrap();
        let prepared = BoundedProcess::prepare(&binding, "test", 4096).unwrap();
        let refused = prepared
            .execute(|| Err("stale_resource_fence".into()))
            .unwrap_err();
        assert_eq!(refused, "stale_resource_fence");
        let result = BoundedProcess::prepare(&binding, "test", 4096)
            .unwrap()
            .execute(|| Ok(()))
            .unwrap();
        assert_eq!(result.exit_code, Some(7), "{result:?}");
        assert_eq!(result.stdout.matches("denied").count(), 8, "{result:?}");
        assert!(!result.timed_out);
        assert!(!root.join("project/created").exists());
        assert_eq!(std::fs::read(&outside).unwrap(), b"not a Case resource");
        std::fs::write(root.join("project/test.py"), "while True: pass\n").unwrap();
        let ResourceAddress::ProcessRunner { runners, .. } = &mut binding.address else {
            panic!()
        };
        runners.get_mut("test").unwrap().timeout_ms = 100;
        let timeout = BoundedProcess::prepare(&binding, "test", 4096)
            .unwrap()
            .execute(|| Ok(()))
            .unwrap();
        assert!(
            timeout.timed_out && timeout.elapsed_ms < 2000,
            "{timeout:?}"
        );
        assert_eq!(timeout.signal, Some(libc::SIGKILL));
        std::fs::write(root.join("project/test.py"), "print('x'*100000)\n").unwrap();
        let overflow = BoundedProcess::prepare(&binding, "test", 128)
            .unwrap()
            .execute(|| Ok(()))
            .unwrap();
        assert!(overflow.output_limit_exceeded);
        assert!(overflow.stdout.len() <= 128 && overflow.stderr.len() <= 128);
        println!(
            "bounded_process_component:{}",
            serde_json::json!({"profile":PROCESS_PROFILE,"exit":result.exit_code,
            "external_read_write_network_fork_chmod_prlimit_ioctl":"denied", "timeout_ms":timeout.elapsed_ms,
            "output_bound":overflow.stdout.len(),"provider_mode":"no_provider","product_effect":"not_qualified"})
        );
        std::fs::remove_file(outside).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
