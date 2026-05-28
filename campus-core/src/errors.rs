use thiserror::Error;

/// campus-auto-login 项目的统一错误类型
#[derive(Error, Debug)]
pub enum CampusError {
    // ── 配置文件 ──

    /// 配置文件 config.toml 不存在
    #[error("配置文件不存在: {0}")]
    ConfigNotFound(String),

    /// 配置文件 TOML 反序列化失败（字段缺失、类型错误等）
    #[error("配置文件解析失败")]
    ConfigParse(#[source] toml::de::Error),

    /// 配置文件 TOML 序列化失败
    #[error("配置文件序列化失败")]
    ConfigSerialize(#[source] toml::ser::Error),

    // ── 页面元素配置 ──

    /// 页面元素配置文件 elements.toml 不存在
    #[error("页面元素配置文件不存在: {0}")]
    ElementsNotFound(String),

    /// 页面元素配置文件 TOML 解析失败
    #[error("页面元素配置文件解析失败")]
    ElementsParse(#[source] toml::de::Error),

    // ── Chrome / ChromeDriver ──

    /// Chrome 浏览器可执行文件未找到
    #[error("Chrome 浏览器未找到: {0}")]
    ChromeNotFound(String),

    /// ChromeDriver 可执行文件未找到
    #[error("ChromeDriver 未找到: {0}")]
    ChromeDriverNotFound(String),

    /// ChromeDriver 子进程启动失败
    #[error("ChromeDriver 启动失败")]
    ChromeDriverStart(#[source] std::io::Error),

    /// Chrome WebDriver 连接失败（端口不通、协议错误等）
    #[error("Chrome WebDriver 连接失败: {0}")]
    ChromeConnection(String),

    // ── 网络 ──

    /// HTTP 网络请求失败（DNS 解析、连接超时、TLS 错误等）
    #[error("网络请求失败")]
    NetworkRequest(#[source] reqwest::Error),

    /// 登录操作失败（无法区分具体原因时使用）
    #[error("登录失败: {0}")]
    LoginFailed(String),

    // ── 进程管理 ──

    /// 文件锁（lockfile）操作失败
    #[error("文件锁操作失败")]
    LockError(#[source] std::io::Error),

    /// 已有另一个 auto-login 实例在运行
    #[error("实例已在运行")]
    AlreadyRunning,

    /// IPC 通信失败（停止信号无法发送/接收）
    #[error("IPC 连接失败: {0}")]
    IpcConnection(String),

    // ── 通用 I/O ──

    /// 通用 I/O 错误（文件读写、目录创建等）
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),
}


