// 下载字体的 browser 本地回退消费：页面与浏览器 chrome 不共享 renderer 的数字 ID。
impl BrowserApp {
    fn sync_page_fonts(&mut self) -> bool {
        let fonts = self
            .shell
            .active_tab_id()
            .and_then(|id| self.tabs.snapshot(id))
            .map(|snap| snap.font_payloads.as_slice())
            .unwrap_or(&[]);
        let ids: std::collections::HashSet<_> = fonts.iter().map(|font| font.font_id).collect();
        if ids.len() != fonts.len() {
            return false;
        }
        if self.page_font_imports.len() == fonts.len()
            && fonts.iter().all(|font| {
                self.page_font_imports.get(&font.font_id).is_some_and(|id| {
                    self.font_loader.face_index(*id) == font.face_index
                        && self.font_loader.get_font_data(*id) == Some(font.data.as_slice())
                })
            })
        {
            return true;
        }

        // 复用 compositor 的资源信任边界；无效输入不能替换当前可用 registry。
        let (base, _) = Self::cached_system_fonts();
        let mut validated = zero_paint_convert::fonts::PaintFonts::new(std::sync::Arc::new(base));
        if let Err(error) = validated.update(fonts) {
            tracing::warn!(%error, "browser: rejected page font resources");
            return false;
        }
        let mut loader = self.font_loader.duplicate();
        for id in self.page_font_imports.values() {
            loader.remove_font(*id);
        }
        let mut mapping = HashMap::new();
        for font in fonts {
            let Ok(id) = loader.load_font_at_index(&font.data, font.face_index) else {
                return false;
            };
            mapping.insert(font.font_id, id);
        }
        self.font_loader = loader;
        self.page_font_imports = mapping;
        self.glyph_cache.clear();
        if let Some(gpu) = self.gpu_renderer.as_mut() {
            gpu.clear_glyph_atlas();
        }
        self.retained_fb = None;
        true
    }
}

#[cfg(test)]
mod page_font_tests {
    use super::*;

    #[test]
    fn page_fonts_do_not_replace_chrome_fonts_and_are_released_on_navigation() {
        let mut app = BrowserApp::new(RenderMode::Cpu);
        let tab = app.shell.active_tab_id().unwrap();
        app.tabs.ensure_snapshot_for_test(tab);
        let ui_id = app.font_id.unwrap();
        let ui_data = app.font_loader.get_font_data(ui_id).unwrap().to_vec();
        let font = zero_protocol::IpcFontPayload {
            font_id: 420,
            face_index: 0,
            data: include_bytes!("../../../tests/wpt-runner/fonts/Ahem.ttf").to_vec(),
        };
        app.tabs.snapshot_mut(tab).unwrap().font_payloads = vec![font.clone()];
        assert!(app.sync_page_fonts());
        let imported = app.page_font_imports[&420];
        assert_ne!(imported, ui_id);
        assert_eq!(app.font_loader.measure_advance(imported, 'X', 40.0), 40.0);
        assert_eq!(app.font_loader.get_font_data(ui_id), Some(ui_data.as_slice()));
        app.tabs.snapshot_mut(tab).unwrap().font_payloads = vec![font.clone(), font];
        assert!(!app.sync_page_fonts(), "duplicate IDs must fail closed");
        app.tabs
            .snapshot_mut(tab)
            .unwrap()
            .begin_navigation("about:blank".into());
        assert!(app.sync_page_fonts());
        assert!(app.page_font_imports.is_empty());
        assert!(app.font_loader.get_font_data(imported).is_none());
        assert_eq!(app.font_loader.get_font_data(ui_id), Some(ui_data.as_slice()));
    }
}
