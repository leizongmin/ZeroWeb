import native from 'libzeroui.so';

@Entry
@Component
struct Index {
  @State message: string = 'ZeroBrowser';

  build() {
    Column() {
      Text(this.message)
        .fontSize(30)
        .fontWeight(FontWeight.Bold)
        .margin({ top: 40 })
    }
    .width('100%')
    .height('100%')
    .onTouch((event: TouchEvent) => {
      if (event.type === TouchType.Down) {
        for (let i = 0; i < event.touches.length; i++) {
          const t = event.touches[i];
          native.dispatchTouch(t.id, t.x, t.y, 0); // 0=ACTION_DOWN
        }
      } else if (event.type === TouchType.Move) {
        for (let i = 0; i < event.touches.length; i++) {
          const t = event.touches[i];
          native.dispatchTouch(t.id, t.x, t.y, 2); // 2=ACTION_MOVE
        }
      } else if (event.type === TouchType.Up) {
        for (let i = 0; i < event.touches.length; i++) {
          const t = event.touches[i];
          native.dispatchTouch(t.id, t.x, t.y, 1); // 1=ACTION_UP
        }
      }
    })
    .onKeyEvent((event: KeyEvent) => {
      // Key events are routed through the window stage callback in EntryAbility
      return false;
    })
    .onAppear(() => {
      if (native.isRuntimeReady()) {
        this.message = 'ZeroBrowser Ready';
      }
    })
  }
}
