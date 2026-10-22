// 窗口插件：网关有了结果（起来了或退回 file://）之后开托盘和主窗口。
// 退出时先把所有窗口藏起来，界面上看起来已经退出；后端子进程随后在后端插件里收尾。
function createWindowsPlugin({ app, BrowserWindow, desktopWindows, onStartupFailure }) {
  function windows(ctx) {
    const { createTray, createWindow, hasLiveMainWindow, showMainWindow } = desktopWindows;
    ctx.effect(() => () => {
      for (const window of BrowserWindow.getAllWindows()) {
        if (!window.isDestroyed()) {
          window.hide();
        }
      }
    }, "hide windows on quit");
    try {
      createTray();
      createWindow();
    } catch (error) {
      onStartupFailure(error);
      return;
    }
    const onActivate = () => {
      if (!hasLiveMainWindow()) {
        createWindow();
        return;
      }
      showMainWindow();
    };
    ctx.effect(() => {
      app.on("activate", onActivate);
      return () => app.removeListener("activate", onActivate);
    }, "app activate");
  }
  windows.inject = ["gateway"];
  return windows;
}

module.exports = { createWindowsPlugin };
