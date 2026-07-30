# SeaLantern Connect

SeaLantern Connect 是与 [SeaLantern](https://github.com/SeaLantern-Studio/SeaLantern) 配套的轻量联机客户端。

玩家可以粘贴由 SeaLantern 生成的联机邀请，与房主建立 P2P 连接。连接成功后，远端世界会出现在 Minecraft 的局域网服务器列表中，也可以使用软件显示的本地地址手动加入。

无需公网 IP，也无需配置路由器端口转发。

## 开发

本项目由 [only](https://github.com/SeaLantern-Studio/SeaLantern-Connect) 提供开发便捷命令，安装详见 [这里](https://github.com/KercyDing/only#install)。

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

注意，commit 之前请运行本地 CI 测试：

```bash
only ci
```

## 许可证

[GNU Affero General Public License v3.0](LICENSE)
