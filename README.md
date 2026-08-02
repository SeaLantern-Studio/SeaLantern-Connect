<div align="center">

<img src="src/assets/logo.png" alt="logo" width="200" height="200">

## 海晶互联（SeaLantern-Connect）

专为 [SeaLantern](https://github.com/SeaLantern-Studio/SeaLantern) 打造的轻量联机客户端

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
</div>

<kbd>简体中文</kbd> <kbd>[English](README-en.md)</kbd>

</div>

## 能干什么

让 Minecraft Java 版联机更简单：创建房间、分享邀请，然后一起进入世界。

> 无需公网 IP，也无需手动设置端口转发。

## 软件特色

- **轻松开房**：开启局域网世界后，即可快速创建联机房间。
- **链接邀请**：分享邀请链接，朋友打开后便可加入。
- **稳定连接**：自动处理连接、断线恢复和状态提醒。
- **轻量运行**：暂时不用时可安静地留在后台。
- **原生质感**：支持 Windows 云母/亚克力和 macOS 毛玻璃/液态玻璃效果。
- **随心定制**：提供中英文界面、明暗模式、主题配色和字体设置。

## 给开发者

本项目使用 [only](https://github.com/KercyDing/only) 作为开发工具链，安装详见 [这里](https://github.com/KercyDing/only#install)。

### 常用命令

启动开发模式：

```bash
only dev
```

启用 DEBUG 等级日志：

```bash
only dev debug
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
