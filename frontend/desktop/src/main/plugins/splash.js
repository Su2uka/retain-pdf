// 启动画面插件：开窗、加载 splash.html，提供 `splash` 服务 { update, close, getWindow }。
// 后端和网关插件靠它报启动进度；主窗口 ready-to-show 时经 close() 关掉它。
const path = require("path");

function createSplashPlugin({ BrowserWindow, appRoot, resolveWindowIcon, onStartupFailure }) {
  async function splash(ctx) {
    let win = null;
    const service = {
      update(progress, title, detail) {
        if (!win || win.isDestroyed()) {
          return;
        }
        win.webContents.send("startup-progress", { progress, title, detail });
      },
      close() {
        if (win && !win.isDestroyed()) {
          win.close();
        }
        win = null;
      },
      getWindow() {
        return win && !win.isDestroyed() ? win : null;
      },
    };
    ctx.effect(() => () => service.close(), "splash window");

    try {
      win = new BrowserWindow({
        width: 520,
        height: 360,
        frame: false,
        resizable: false,
        maximizable: false,
        minimizable: false,
        fullscreenable: false,
        autoHideMenuBar: true,
        center: true,
        backgroundColor: "#f5f5f7",
        icon: resolveWindowIcon(),
        webPreferences: {
          preload: path.join(appRoot, "preload.js"),
          contextIsolation: true,
          nodeIntegration: false,
          sandbox: false,
        },
      });
      await win.loadFile(path.join(appRoot, "splash.html"));
    } catch (error) {
      onStartupFailure(error);
      return;
    }
    service.update(6, "正在准备运行环境", "正在检查桌面组件与本地资源");
    ctx.provide("splash", service);
  }
  return splash;
}

module.exports = { createSplashPlugin };
