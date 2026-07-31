<div align="center">

<img src="src/assets/logo.png" alt="logo" width="200" height="200">

## 海晶互联（SeaLantern-Connect）

专为 [SeaLantern](https://github.com/SeaLantern-Studio/SeaLantern) 打造的轻量联机客户端

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
</div>

<kbd>简体中文</kbd> <kbd>[English](README-en.md)</kbd>

</div>

## 能干什么

玩家只需粘贴 SeaLantern 生成的联机邀请，即可与房主建立 P2P 连接。

连接成功后，远端世界将自动出现在 Minecraft 的局域网服务器列表中；你也可以使用软件显示的本地地址手动加入。

> 无需公网 IP，无需配置路由器端口转发。

## 开发

本项目使用 [only](https://github.com/KercyDing/only) 作为开发工具链，安装详见[这里](https://github.com/KercyDing/only#install)。

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

## 许可证

[GNU Affero General Public License v3.0](LICENSE)
