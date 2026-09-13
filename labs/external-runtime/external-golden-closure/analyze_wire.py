#!/usr/bin/env python3
"""Read an opt-in strace capture; never invoke/retry a provider.

Requires `strace -f -s 131072 -xx -yy -e trace=network`. Only complete
sendto/recvfrom calls on the selected public TCP destination are accepted.
Inputs may contain Case data: use fresh synthetic qualification homes only.
--allow-no-response preserves a fully sent request with zero received bytes;
it never fabricates an HTTP status or accepts a partially received response.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


def encoded(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def field_sizes(value):
    # Each size includes field name, colon and value, excludes object commas.
    return {k: len(encoded(k)) + 1 + len(encoded(v)) for k, v in value.items()}


def escaped_fields(value):
    return {k: len(encoded((encoded(k) + b":" + encoded(v)).decode())) - 2 for k, v in value.items()}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("trace", type=Path)
    p.add_argument("--destination", required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--home", type=Path, help="fresh retained qualification home only; read-only mdb_dump")
    p.add_argument("--allow-no-response", action="store_true", help="retain zero response bytes as unknown, not success")
    a = p.parse_args()
    a.output.mkdir(exist_ok=False)
    artifacts = {}
    if a.home:
        dump = subprocess.check_output(["mdb_dump", "-s", "semantic_context_artifacts", str(a.home / "store/lmdb")])
        rows = dump.split(b"HEADER=END\n", 1)[1].split(b"DATA=END", 1)[0].splitlines()
        assert len(rows) % 2 == 0
        for key, data in zip(rows[::2], rows[1::2]):
            raw = bytes.fromhex(data.decode().strip())
            wrapper = json.loads(raw)
            assert encoded(wrapper) == raw
            identity = bytes.fromhex(key.decode().strip()).decode().removeprefix("semantic-context:id:")
            artifacts[identity] = wrapper["artifact"]
    streams = {}
    pattern = re.compile(r'\d+ (sendto|recvfrom)\(\d+<TCP:\[([^]]+)\]>, "((?:\\x[0-9a-f]{2})*)", .*\) = (\d+)$')
    for line in a.trace.read_text().splitlines():
        if "->" + a.destination + "]" not in line:
            continue
        if "sendto(" not in line and "recvfrom(" not in line:
            continue
        if "= -1 EAGAIN" in line or "= -1 EINTR" in line:
            continue
        m = pattern.fullmatch(line)
        if not m:
            raise ValueError("incomplete/unsupported syscall: " + line[:160])
        direction, socket, hexes, count = m.groups()
        data = bytes.fromhex(hexes.replace("\\x", ""))
        assert len(data) >= int(count), "truncated strace payload"
        stream = streams.setdefault(socket, {"sendto": bytearray(), "recvfrom": bytearray()})
        stream[direction].extend(data[:int(count)])
    summary = []
    for order, (socket, stream) in enumerate(streams.items(), 1):
        request = bytes(stream["sendto"])
        response = bytes(stream["recvfrom"])
        if not request:
            continue
        header, body = request.split(b"\r\n\r\n", 1)
        headers = dict(line.lower().split(b": ", 1) for line in header.split(b"\r\n")[1:])
        assert int(headers.get(b"content-length", b"0")) == len(body)
        if response:
            response_header, response_body = response.split(b"\r\n\r\n", 1)
            response_headers = dict(line.lower().split(b": ", 1) for line in response_header.split(b"\r\n")[1:])
            assert int(response_headers[b"content-length"]) == len(response_body)
        else:
            assert a.allow_no_response, "zero response bytes; explicit --allow-no-response required"
            response_header = response_body = None
        record = dict(order=order, socket=socket, request_line=header.split(b"\r\n")[0].decode(),
                      http_request_bytes=len(request), header_bytes=len(header)+4,
                      body_bytes=len(body), body_sha256=hashlib.sha256(body).hexdigest(),
                      response_header=response_header.decode() if response_header is not None else None,
                      response_body=response_body.decode() if response_body is not None else None,
                      response_bytes=len(response))
        if body:
            value = json.loads(body)
            assert len(encoded(value)) == len(body), "not the observed compact encoding"
            record["top_level_field_bytes"] = field_sizes(value)
            record["message_content_utf8_bytes"] = [len(m["content"].encode()) if isinstance(m.get("content"), str) else len(encoded(m.get("content"))) for m in value["messages"]]
            record["message_content_json_bytes"] = [len(encoded(m.get("content"))) for m in value["messages"]]
            frames = [m["content"] for m in value["messages"] if isinstance(m.get("content"), str) and m["content"].startswith("YAI typed ContextFrame:\n")]
            if frames:
                assert len(frames) == 1
                frame_text = frames[0].split("\n", 1)[1]
                frame = json.loads(frame_text)
                assert encoded(frame).decode() == frame_text
                record["context_frame_bytes"] = len(frame_text.encode())
                record["context_frame_fields"] = field_sizes(frame)
                record["context_frame_fields_escaped_in_message"] = escaped_fields(frame)
                assert sum(record["context_frame_fields_escaped_in_message"].values()) + len(frame) + 1 + len(encoded("YAI typed ContextFrame:\n")) == len(encoded(frames[0]))
                record["entry_bytes"] = [dict(id=e["entry_id"], bytes=len(encoded(e)), posture=e["posture"], value_bytes=len(encoded(e["value"]))) for e in frame["entries"]]
                record["output_contract_fields"] = field_sizes(frame["output_contract"].get("contract", {}))
                record["tools_count"] = len(value.get("tools", []))
                view = frame["output_contract"].get("contract", {}).get("view")
                if view:
                    resources = [encoded(e["resource"]) for e in view["entries"]]
                    contributions = [encoded(c) for e in view["entries"] for constraint in e["policy_constraints"] for c in constraint.get("contributions", [])]
                    record["capability_view_fields"] = field_sizes(view)
                    record["capability_resource_objects"] = dict(occurrences=len(resources), unique=len(set(resources)), occurrence_bytes=sum(map(len, resources)), unique_bytes=sum(map(len, set(resources))))
                    record["policy_contribution_objects"] = dict(occurrences=len(contributions), unique=len(set(contributions)), occurrence_bytes=sum(map(len, contributions)), unique_bytes=sum(map(len, set(contributions))))
                if artifacts:
                    assert artifacts[frame["frame_id"]] == frame
                    projection = artifacts[frame["projection_id"]]
                    working = artifacts[projection["bounds"]["working_state_id"]]
                    assert working["entries"] == projection["entries"] == frame["entries"]
                    record["working_state_bytes"] = len(encoded(working))
                    record["working_state_fields"] = field_sizes(working)
                    record["working_state_bounds"] = working["bounds"]
                    record["projection_bytes"] = len(encoded(projection))
                    record["projection_fields"] = field_sizes(projection)
                    record["projection_bounds"] = projection["bounds"]
                    record["entries_exactly_equal_at_all_three_boundaries"] = True
                    for name, artifact in [("working", working), ("projection", projection)]:
                        (a.output / f"{order:02d}-{name}.json").write_bytes(encoded(artifact))
                (a.output / f"{order:02d}-http.json").write_text(json.dumps({"request_header": header.decode() + "\r\n\r\n", "response": response.decode() if response else None}, indent=2) + "\n")
                (a.output / f"{order:02d}-body.json").write_bytes(body)
                (a.output / f"{order:02d}-frame.json").write_text(frame_text)
        summary.append(record)
    (a.output / "wire.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
