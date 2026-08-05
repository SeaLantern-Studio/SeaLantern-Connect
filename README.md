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
- **随心定制**：提供中英文界面、明暗模式、自定义主题配色和字体设置。

## 凭证存储的安全边界

FRP 凭证会以认证加密格式保存在本地，而不是以明文写入文件。这主要用于降低备份、同步或临时文件意外暴露明文凭证的风险。

该保护不是操作系统密钥链：加密密钥随应用程序编译，能够取得安装包并分析二进制的人可以恢复密钥并解密凭证。因此它不能防御本机攻击者、恶意软件或已有用户权限的程序。未来可通过凭证格式版本迁移到 Windows DPAPI、macOS Keychain 或 Linux Secret Service 等平台安全存储。

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

Arch Linux 及其衍生发行版可在本地构建 pacman 包并直接安装：

```bash
only arch install
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

[Apache License 2.0](LICENSE)
