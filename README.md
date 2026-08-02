<div align="center">

<img src="src/assets/logo.png" alt="logo" width="200" height="200">

## 海晶互联（SeaLantern-Connect）

专为 [SeaLantern](https://github.com/SeaLantern-Studio/SeaLantern) 打造的轻量联机客户端

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
</div>

<kbd>简体中文</kbd> <kbd>[English](README-en.md)</kbd>

</div>

## 能干什么

SeaLantern Connect 是一个让 Minecraft 多人联机更简单的工具。

无需公网 IP，无需端口转发，只需开启局域网联机并分享邀请链接，朋友即可快速加入你的世界。

支持邀请管理、自动连接、断线恢复、状态监控、轻量运行，以及中英文界面和主题自定义。

让 Minecraft 联机回归简单：创建房间，分享链接，开始游戏。

## 给开发者

本项目使用 [only](https://github.com/KercyDing/only) 作为开发工具链，安装详见 [这里](https://github.com/KercyDing/only#install)。

### 常用命令

启动开发模式：

```bash
only dev
```

构建应用：

```bash
only build
```

### 本地 CI 测试

提交代码前，请先运行本地 CI 测试：

```bash
only ci
```

### Deep Link 开发测试

开发模式会在 Windows 和 Linux 上注册 `sculk` 协议。请使用真实房间邀请测试完整流程。

macOS 只能通过安装到 `/Applications` 的已打包应用测试协议唤起。

## 许可证

[GNU Affero General Public License v3.0](LICENSE)
