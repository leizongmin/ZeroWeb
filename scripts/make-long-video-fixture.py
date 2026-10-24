#!/usr/bin/env python3
"""长 VP9 基线 fixture 生成（media-runtime-isolation M1）。

把仓内 2s fixture 的簇（Cluster）序列整段重复 N 份拼成长流，重建 Cues 与
Duration——零外部素材、零编码器依赖，来源清白可重现（母本 = tests/fixtures/media/
既有 ffmpeg 产物）。每份簇序列以 keyframe 起始，重复后仍是合法 VP9 参考链。

用法：python3 scripts/make-long-video-fixture.py <输入.webm> <输出.webm> <重复次数>
"""

import struct
import sys

# ── EBML 元素 ID（仅本脚本需要识别的子集）───────────────────────────────

ID_EBML = 0x1A45DFA3
ID_SEGMENT = 0x18538067
ID_VOID = 0xEC
ID_INFO = 0x1549A966
ID_CLUSTER = 0x1F43B675
ID_TRACKS = 0x1654AE6B
ID_CUES = 0x1C53BB6B
ID_DURATION = 0x4489
ID_TIMECODE = 0xE7
ID_SIMPLEBLOCK = 0xA3
ID_BLOCKGROUP = 0xA0


def read_vint(buf, pos, keep_marker):
    """读 EBML varint。keep_marker=True（元素 ID）保留首字节标记位；
    False（尺寸）剥离长度前缀位。返回 (value, new_pos)。"""
    first = buf[pos]
    if first == 0:
        raise ValueError("varint 首字节全零（非法）")
    length = 1
    mask = 0x80
    while not (first & mask):
        mask >>= 1
        length += 1
    value = first if keep_marker else (first & (mask - 1))
    for i in range(1, length):
        value = (value << 8) | buf[pos + i]
    return value, pos + length


def encode_vint_size(value):
    """尺寸 varint：最短形式。长度标记位 = 值域高位外的 1（value + 2^(7L)，
    全 1 保留给 unknown size，故无进位冲突）。"""
    for length in range(1, 9):
        cap = (1 << (7 * length)) - 1
        if value < cap:
            return (value + (1 << (7 * length))).to_bytes(length, "big")
    raise ValueError("size varint overflow")


def encode_uint(value):
    if value == 0:
        return b"\x00"
    n = (value.bit_length() + 7) // 8
    return value.to_bytes(n, "big")


def encode_elem(eid_bytes, payload):
    return eid_bytes + encode_vint_size(len(payload)) + payload


def parse_children(buf, start, end):
    """解析 [start, end) 内的 (id, data_start, data_end, total_start, total_end)。"""
    out = []
    pos = start
    while pos < end:
        elem_start = pos
        eid, pos = read_vint(buf, pos, keep_marker=True)
        size, pos = read_vint(buf, pos, keep_marker=False)
        data_start = pos
        data_end = end if size == (1 << 56) - 1 else data_start + size
        out.append((eid, data_start, data_end, elem_start, data_end))
        pos = data_end
    return out


def main():
    if len(sys.argv) != 4:
        print(__doc__)
        sys.exit(2)
    src_path, dst_path, copies = sys.argv[1], sys.argv[2], int(sys.argv[3])
    with open(src_path, "rb") as f:
        buf = f.read()

    top = parse_children(buf, 0, len(buf))
    ebml_hdr = next((e for e in top if e[0] == ID_EBML), None)
    segment = next((e for e in top if e[0] == ID_SEGMENT), None)
    if ebml_hdr is None or segment is None:
        sys.exit("输入文件缺 EBML header 或 Segment")
    ebml_hdr_bytes = buf[ebml_hdr[3]:ebml_hdr[4]]

    children = parse_children(buf, segment[1], segment[2])
    info = next((e for e in children if e[0] == ID_INFO), None)
    tracks = next((e for e in children if e[0] == ID_TRACKS), None)
    clusters = [e for e in children if e[0] == ID_CLUSTER]
    if info is None or tracks is None or not clusters:
        sys.exit("输入文件缺 Info/Tracks/Cluster")

    # 时长与 timebase：Duration（tick）× TimestampScale(ns) → ms；簇偏移以 ms 计
    # （母本 scale=1e6 → 1 tick = 1 ms）。
    info_children = parse_children(buf, info[1], info[2])
    dur_elem = next((e for e in info_children if e[0] == ID_DURATION), None)
    scale_elem = next((e for e in info_children if e[0] == 0x2AD7B1), None)
    if dur_elem is None:
        sys.exit("Info 缺 Duration")
    scale = 1_000_000
    if scale_elem is not None:
        scale = int.from_bytes(buf[scale_elem[1]:scale_elem[2]], "big")
    if scale != 1_000_000:
        sys.exit(f"仅支持 scale=1e6 的母本（当前 {scale}）——簇偏移按 ms 处理")
    raw_dur = buf[dur_elem[1]:dur_elem[2]]
    clip_dur = struct.unpack(">f", raw_dur)[0] if len(raw_dur) == 4 else struct.unpack(">d", raw_dur)[0]
    clip_ms = round(clip_dur * scale / 1e6)
    print(f"source: clip duration {clip_ms} ms, clusters {len(clusters)}")

    # 解析每簇：Timecode + 块表（track / rel_tc / keyframe）；SimpleBlock 原样透传。
    parsed_clusters = []
    for c in clusters:
        tc = None
        has_keyframe = False
        for ch in parse_children(buf, c[1], c[2]):
            if ch[0] == ID_TIMECODE:
                tc = int.from_bytes(buf[ch[1]:ch[2]], "big")
            elif ch[0] == ID_SIMPLEBLOCK:
                b = buf[ch[1]:ch[2]]
                _track, p2 = read_vint(b, 0, keep_marker=False)
                _rel = b[p2:p2 + 2]
                if b[p2 + 2] & 0x80:
                    has_keyframe = True
            # BlockGroup（参考块 → 非 keyframe）：原样透传，不参与 cue。
        if tc is None:
            sys.exit("簇缺 Timecode")
        parsed_clusters.append((tc, has_keyframe))
    base_tc = parsed_clusters[0][0]
    if not parsed_clusters[0][1]:
        sys.exit("首簇无 keyframe——重复拼接的参考链前提不成立")

    # ── 重组输出：Info（patch Duration）+ Tracks + 簇副本 + 重建 Cues ──
    info_bytes = bytearray()
    for ch in info_children:
        if ch[0] == ID_DURATION:
            # 保持母本 4 字节 float 形态（encode float32）。
            info_bytes += encode_elem(bytes([0x44, 0x89]), struct.pack(">f", float(clip_ms * copies)))
        else:
            info_bytes += buf[ch[3]:ch[4]]
    tracks_bytes = buf[tracks[3]:tracks[4]]

    pre = len(encode_elem(bytes([0x15, 0x49, 0xA9, 0x66]), bytes(info_bytes))) + len(tracks_bytes)
    cluster_blobs = []
    first_cluster_pos = []
    pos = pre
    for i in range(copies):
        offset = i * clip_ms
        for ci, (tc, _kf) in enumerate(parsed_clusters):
            tc_payload = encode_uint(tc - base_tc + offset)
            body = encode_elem(bytes([ID_TIMECODE]), tc_payload)
            # 簇内其余子元素（SimpleBlock/BlockGroup 等）原样透传。
            for ch in parse_children(buf, clusters[ci][1], clusters[ci][2]):
                if ch[0] == ID_TIMECODE:
                    continue
                body += buf[ch[3]:ch[4]]
            elem = encode_elem(bytes([0x1F, 0x43, 0xB6, 0x75]), body)
            if ci == 0:
                first_cluster_pos.append(pos)
            cluster_blobs.append(elem)
            pos += len(elem)

    # Cues：每副本首簇位置一个 cue 点（首簇带 keyframe；CueTrack=视频轨 1——
    # 母本单视频轨或双轨首轨，matroska_demuxer seek 二分只需点位正确）。
    # 簇位置以 Segment data start 起算，SeekHead 前移量在下方两遍收敛后统一补。
    cues_elem = b""
    seg_body_core = (
        encode_elem(bytes([0x15, 0x49, 0xA9, 0x66]), bytes(info_bytes))
        + tracks_bytes
        + b"".join(cluster_blobs)
    )
    seekhead_elem = b""

    # SeekHead（matroska_demuxer 的 seek 硬依赖 Cluster 条目——Cue 二分与
    # find_first_cluster_offset 都从它取基准）。条目位置相对 Segment data start。
    # 置于段首会前移后续元素——条目偏移随之变化，两遍收敛（长度档位稳定即止）。
    info_len = len(encode_elem(bytes([0x15, 0x49, 0xA9, 0x66]), bytes(info_bytes)))
    tracks_len = len(tracks_bytes)
    clusters_len = sum(len(c) for c in cluster_blobs)

    def build_seekhead(shift):
        base = shift
        entries = [
            (bytes([0x15, 0x49, 0xA9, 0x66]), base),  # Info
            (bytes([0x16, 0x54, 0xAE, 0x6B]), base + info_len),  # Tracks
            (bytes([0x1F, 0x43, 0xB6, 0x75]), base + info_len + tracks_len),  # Cluster
            (bytes([0x1C, 0x53, 0xBB, 0x6B]), base + info_len + tracks_len + clusters_len),  # Cues
        ]
        body = bytearray()
        for eid, offset in entries:
            body += encode_elem(
                bytes([0x4D, 0xBB]),
                encode_elem(bytes([0x53, 0xAB]), eid)
                + encode_elem(bytes([0x53, 0xAC]), encode_uint(offset)),
            )
        return encode_elem(bytes([0x11, 0x4D, 0x9B, 0x74]), bytes(body))

    sh_len = 0
    for _ in range(4):
        cand = build_seekhead(sh_len)
        if len(cand) == sh_len:
            seekhead_elem = cand
            break
        sh_len = len(cand)
    assert len(seekhead_elem) == sh_len, "SeekHead 尺寸未收敛"

    # cue 簇位置统一补 SeekHead 前移量后重建 Cues。
    cues_body = bytearray()
    for i, cpos in enumerate(first_cluster_pos):
        ct = encode_uint(i * clip_ms)
        trk = encode_uint(1)
        cp = encode_uint(cpos + sh_len)
        ctp = encode_elem(bytes([0xF7]), trk) + encode_elem(bytes([0xF1]), cp)
        cue = encode_elem(bytes([0xB3]), ct) + encode_elem(bytes([0xB7]), ctp)
        cues_body += encode_elem(bytes([0xBB]), cue)
    cues_elem = encode_elem(bytes([0x1C, 0x53, 0xBB, 0x6B]), bytes(cues_body))

    seg_payload = seekhead_elem + seg_body_core + cues_elem
    # Segment 尺寸用 8 字节定长 varint（0x01 前缀），位置无关。
    assert len(seg_payload) < (1 << 56) - 1
    seg_size = bytes([0x01]) + len(seg_payload).to_bytes(7, "big")
    with open(dst_path, "wb") as f:
        f.write(ebml_hdr_bytes)
        f.write(bytes([0x18, 0x53, 0x80, 0x67]) + seg_size)
        f.write(seg_payload)
    print(f"wrote {dst_path}: copies={copies} duration={clip_ms * copies} ms cues={len(first_cluster_pos)}")


if __name__ == "__main__":
    main()
