import { AbilityConstant, UIAbility, Want } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { window } from '@kit.ArkUI';
import native from 'libzeroui.so';

export default class EntryAbility extends UIAbility {
  private renderTaskId: number = -1;
  private running: boolean = false;

  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(0x3200, 'ZeroBrowser', 'EntryAbility onCreate');
  }

  onDestroy(): void {
    this.running = false;
    if (this.renderTaskId >= 0) clearInterval(this.renderTaskId);
    native.shutdown();
    hilog.info(0x3200, 'ZeroBrowser', 'EntryAbility onDestroy');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    hilog.info(0x3200, 'ZeroBrowser', 'onWindowStageCreate');

    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(0x3200, 'ZeroBrowser', 'Failed to load content: %{public}d', err.code);
        return;
      }
      this.initRuntime(windowStage);
    });
  }

  private initRuntime(windowStage: window.WindowStage): void {
    const mainWindow = windowStage.getMainWindowSync();
    const rect = mainWindow.getWindowProperties().windowRect;
    const density = mainWindow.getWindowProperties().densityDPI / 160;

    native.initRuntime(rect.width, rect.height, density);
    native.windowSizeChange(rect.width, rect.height, density,
      1.0, // textScale
      0, 0, 0, 0, // safeArea (top/right/bottom/left, system bars handled by ArkUI)
      rect.width > rect.height ? 0 : 1); // portrait=1

    this.running = true;

    // Render loop: pump events at ~60fps
    this.renderTaskId = setInterval(() => {
      if (!this.running || !native.isRuntimeReady()) return;
      native.pumpEvents();
    }, 16);
  }

  onWindowStageDestroy(): void {
    this.running = false;
    if (this.renderTaskId >= 0) clearInterval(this.renderTaskId);
  }
}
