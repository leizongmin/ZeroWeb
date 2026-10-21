//! 证据采集辅助：把已合成帧的 front buffer 原样落盘（env 门控，默认关闭）。
//!
//! compositor-animation-continuity goal 的真实窗口基线工具：合成器只在收到
//! renderer 帧时提交（无动画泵时提交序列即「帧生产节奏」的直接证据）。dump 在
//! swap 后读 front buffer，写裸 RGBA + JSONL manifest（含 wall-clock 时间戳），
//! PNG 转换由外部探针完成——本 crate 不引入图像编码依赖。
//!
//! 资源边界：帧数上限 `ZW_COMPOSITOR_FRAME_DUMP_MAX`（默认 32，钳到 1..=256）；
//! 任一写失败即永久停用并 warn 一次——诊断工具不得拖垮合成路径（run-rules
//! 学习 #14：资源工具必须有降级出口）。

use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// env 门控的帧落盘器；未设 `ZW_COMPOSITOR_FRAME_DUMP_DIR` 时为空实现。
pub struct FrameDumper {
    dir: PathBuf,
    max_frames: u32,
    written: u32,
    disabled: bool,
}

impl FrameDumper {
    /// 读 env 构造；未启用返回 `None`。
    pub fn from_env() -> Option<Self> {
        let dir = PathBuf::from(std::env::var("ZW_COMPOSITOR_FRAME_DUMP_DIR").ok()?);
        let max_frames = std::env::var("ZW_COMPOSITOR_FRAME_DUMP_MAX")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(32)
            .clamp(1, 256);
        tracing::info!(
            "compositor: frame dump 已启用 dir={} max_frames={max_frames}",
            dir.display()
        );
        Some(Self {
            dir,
            max_frames,
            written: 0,
            disabled: false,
        })
    }

    /// 落盘一帧 front buffer（RGBA8）。失败降级：warn 一次后永久停用，不阻塞合成。
    pub fn dump(
        &mut self,
        surface_id: u64,
        navigation_epoch: u64,
        frame_id: u64,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) {
        if self.disabled || self.written >= self.max_frames {
            return;
        }
        let result = self.write_frame(surface_id, navigation_epoch, frame_id, width, height, rgba);
        if let Err(error) = result {
            tracing::warn!("compositor: frame dump 写失败，停用本会话采集: {error}");
            self.disabled = true;
        }
    }

    fn write_frame(
        &mut self,
        surface_id: u64,
        navigation_epoch: u64,
        frame_id: u64,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> std::io::Result<()> {
        let wall_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default();
        std::fs::create_dir_all(&self.dir)?;
        let file_name = format!("frame-s{surface_id}-e{navigation_epoch}-f{frame_id}.bin");
        let mut file = std::fs::File::options()
            .write(true)
            .create_new(true)
            .open(self.dir.join(&file_name))?;
        file.write_all(rgba)?;
        file.sync_all()?;
        // manifest 追加写：采样清单与像素一一对应（文件名含 frame_id 天然去重）。
        let mut manifest = std::fs::File::options()
            .create(true)
            .append(true)
            .open(self.dir.join("manifest.jsonl"))?;
        writeln!(
            manifest,
            "{{\"file\":\"{file_name}\",\"surface_id\":{surface_id},\"epoch\":{navigation_epoch},\
             \"frame_id\":{frame_id},\"width\":{width},\"height\":{height},\"bytes\":{},\"wall_ms\":{wall_ms}}}",
            rgba.len()
        )?;
        manifest.sync_all()?;
        self.written += 1;
        if self.written == self.max_frames {
            tracing::info!("compositor: frame dump 达上限 {max}，停止采集", max = self.max_frames);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("zw-comp-dump-test-{}-{tag}", std::process::id()))
    }

    #[test]
    fn dumps_frame_and_manifest_then_stops_at_cap() {
        let dir = dump_dir("cap");
        let _ = std::fs::remove_dir_all(&dir);
        let mut dumper = FrameDumper {
            dir: dir.clone(),
            max_frames: 2,
            written: 0,
            disabled: false,
        };
        dumper.dump(7, 1, 10, 2, 2, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        dumper.dump(7, 1, 11, 2, 2, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
        // 超上限：不写第三帧。
        dumper.dump(7, 1, 12, 2, 2, &[0; 16]);
        assert_eq!(dumper.written, 2);
        let content = std::fs::read_to_string(dir.join("manifest.jsonl")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"frame_id\":10") && lines[0].contains("\"bytes\":16"));
        let pixels = std::fs::read(dir.join("frame-s7-e1-f10.bin")).unwrap();
        assert_eq!(pixels.len(), 16);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_failure_disables_dumper() {
        // 指向一个普通文件路径作为目录 → create_dir_all 失败 → 永久停用。
        let blocker = dump_dir("blocker-file");
        let _ = std::fs::remove_file(&blocker);
        std::fs::write(&blocker, b"not a dir").unwrap();
        let mut dumper = FrameDumper {
            dir: blocker.join("nested"),
            max_frames: 4,
            written: 0,
            disabled: false,
        };
        dumper.dump(1, 1, 1, 1, 1, &[0, 0, 0, 0]);
        assert!(dumper.disabled, "写失败后必须停用（降级不拖垮合成）");
        // 停用后再次调用静默跳过（即使目录问题已消失）。
        std::fs::remove_file(&blocker).unwrap();
        dumper.dump(1, 1, 2, 1, 1, &[0, 0, 0, 0]);
        assert_eq!(dumper.written, 0);
    }
}
