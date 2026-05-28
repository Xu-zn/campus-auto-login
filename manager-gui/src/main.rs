#![windows_subsystem = "windows"]

slint::include_modules!();

use slint::Model;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use std::fs;
use campus_core::config::ConfigFile;
use campus_core::platform;
use campus_core::process::{check_running, query_uptime, start_auto_login, stop_auto_login};

/// 服务商条目（从 elements.toml 解析）
struct ServiceEntry {
    id: String,
    name: String,
}

/// auto-login 进程的 Unix 启动时间戳（秒），从 startup_time 文件读取
static START_TIME: Mutex<Option<u64>> = Mutex::new(None);

/// 全局缓存：已解析的服务商列表（id + name），用于 index ↔ id 映射
static SERVICES: Mutex<Vec<ServiceEntry>> = Mutex::new(Vec::new());

fn main() {
    let manager_window = ManagerWindow::new().expect("manager window出错");

    // ── 运行状态检测 ──
    let weak1 = manager_window.as_weak();
    manager_window.on_check_running_status(move |cur| {
        let window = match weak1.upgrade() {
            Some(w) => w,
            None => return,
        };
        let running = check_running();
        if running {
            let mut t = START_TIME.lock().unwrap();
            if cur != 1 {
                // 首次检测到运行 — 通过 ipmb 查询 auto-login 启动时间戳
                let fallback = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                *t = query_uptime().or(Some(fallback));
            }
            // 每次轮询都刷新运行时长
            let uptime = match *t {
                Some(timestamp) => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let elapsed = now.saturating_sub(timestamp);
                    format_uptime(elapsed)
                }
                None => "--".to_string(),
            };
            drop(t);
            window.set_running_status(1);
            window.set_uptime(uptime.into());
        } else {
            if cur != 2 {
                window.set_running_status(2);
            }
            *START_TIME.lock().unwrap() = None;
            window.set_uptime("--".into());
        }
    });

    // ── 环境检测 ──
    let weak2 = manager_window.as_weak();
    manager_window.on_check_environment(move || {
        let window = match weak2.upgrade() {
            Some(w) => w,
            None => return,
        };

        let config_path = Path::new("config.toml");
        let config_ok = config_path.exists();
        window.set_config_toml_ok(config_ok);
        window.set_config_toml_path(config_path.display().to_string().into());

        let elements_path = Path::new("elements.toml");
        let elements_ok = elements_path.exists();
        window.set_elements_toml_ok(elements_ok);
        window.set_elements_toml_path(elements_path.display().to_string().into());

        let (chrome_ok, chrome_path, driver_ok, driver_path) =
            check_chrome_paths(config_path);

        window.set_chrome_ok(chrome_ok);
        window.set_chrome_path(chrome_path.into());
        window.set_chromedriver_ok(driver_ok);
        window.set_chromedriver_path(driver_path.into());

        // 操作完成后重置 processing 标记（按钮恢复可用，但 if 条件中文件已存在按钮会隐藏）
        if config_ok {
            window.set_env_config_processing(false);
        }
        if elements_ok {
            window.set_env_elements_processing(false);
        }
        if chrome_ok {
            window.set_env_chrome_processing(false);
        }
        if driver_ok {
            window.set_env_driver_processing(false);
        }
    });

    // ── 配置加载 ──
    let config_weak = manager_window.as_weak();
    manager_window.on_load_config(move || {
        let window = match config_weak.upgrade() {
            Some(w) => w,
            None => {
                eprintln!("[config] load: weak upgrade failed");
                return;
            }
        };
        load_config_to_window(&window);
    });

    // ── 配置保存 ──
    let save_weak = manager_window.as_weak();
    manager_window.on_save_config(move || {
        let window = match save_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        save_config_from_window(&window);
    });

    // ── 连接条目增删改 ──
    let conn_weak = manager_window.as_weak();
    manager_window.on_connect_edit(move |action: slint::SharedString| {
        let window = match conn_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let action = action.as_str();
        let entries = window.get_config_connect_entries();
        let Some(model) = entries.as_any().downcast_ref::<slint::VecModel<ConnectEntry>>() else { return };

        if action == "add" {
            model.push(ConnectEntry { url: "".into(), value: "".into() });
        } else if let Some(idx_str) = action.strip_prefix("del-") {
            if model.row_count() > 1 {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if idx < model.row_count() { model.remove(idx); }
                }
            }
        } else if let Some(rest) = action.strip_prefix("url:") {
            // format: "url:INDEX:NEW_VALUE"
            if let Some((idx_str, new_val)) = rest.split_once(':') {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if let Some(mut entry) = model.row_data(idx) {
                        entry.url = new_val.to_string().into();
                        model.set_row_data(idx, entry);
                    }
                }
            }
        } else if let Some(rest) = action.strip_prefix("val:") {
            // format: "val:INDEX:NEW_VALUE"
            if let Some((idx_str, new_val)) = rest.split_once(':') {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if let Some(mut entry) = model.row_data(idx) {
                        entry.value = new_val.to_string().into();
                        model.set_row_data(idx, entry);
                    }
                }
            }
        }
    });

    // ── 启动时自动加载配置 ──
    load_services_to_window(&manager_window);
    load_config_to_window(&manager_window);

    // ── 启动/停止 ──
    manager_window.on_update_running_status(|cur: i32| {
        if cur == 1 {
            stop_auto_login();
        } else {
            // 重置启动时间，下一次轮询时会从 startup_time 文件读取真实启动时间
            *START_TIME.lock().unwrap() = None;
            start_auto_login();
        }
    });

    // ── 环境操作：创建 config.toml ──
    let weak_env_cfg = manager_window.as_weak();
    manager_window.on_env_create_config(move || {
        let window = match weak_env_cfg.upgrade() {
            Some(w) => w,
            None => return,
        };
        window.set_env_config_processing(true);
        let path = Path::new("config.toml");
        if !path.exists() {
            if let Err(e) = ConfigFile::create_default_config(&std::env::current_dir().unwrap_or_default()) {
                eprintln!("[env] 创建 config.toml 失败: {}", e);
            }
        }
        // 定时器下次轮询时会检测到文件存在并刷新 ok 状态
    });

    // ── 环境操作：创建 elements.toml ──
    let weak_env_elem = manager_window.as_weak();
    manager_window.on_env_create_elements(move || {
        let window = match weak_env_elem.upgrade() {
            Some(w) => w,
            None => return,
        };
        window.set_env_elements_processing(true);
        let path = Path::new("elements.toml");
        if !path.exists() {
            let content = "\
[[service]]\nid = \"_service_0\"\nname = \"校园网\"\n\n\
[[service]]\nid = \"_service_1\"\nname = \"中国移动\"\n\n\
[[service]]\nid = \"_service_2\"\nname = \"中国电信\"\n\n\
[[service]]\nid = \"_service_3\"\nname = \"中国联通\"\n\n\
[page]\n\
login_button = \"loginLink_div\"\n\
username_tip = \"username\"\n\
username_input = \"username\"\n\
password_tip = \"pwd_tip\"\n\
password_input = \"pwd\"\n\
service_tip = \"selectDisname\"\n";
            if let Err(e) = fs::write(path, content) {
                eprintln!("[env] 创建 elements.toml 失败: {}", e);
            }
        }
    });

    // ── 环境操作：下载 Chrome ──
    let weak_env_chrome = manager_window.as_weak();
    manager_window.on_env_download_chrome(move || {
        let window = match weak_env_chrome.upgrade() {
            Some(w) => w,
            None => return,
        };
        window.set_env_chrome_processing(true);
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_default();
        let cli_path = exe_dir.join(if cfg!(windows) { "manager-cli.exe" } else { "manager-cli" });
        let _ = std::process::Command::new(&cli_path)
            .args(["download", "--chrome", "-d"])
            .spawn();
        // 定时器下次轮询时会检测到文件存在并刷新 ok 状态
    });

    // ── 环境操作：下载 ChromeDriver ──
    let weak_env_driver = manager_window.as_weak();
    manager_window.on_env_download_driver(move || {
        let window = match weak_env_driver.upgrade() {
            Some(w) => w,
            None => return,
        };
        window.set_env_driver_processing(true);
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_default();
        let cli_path = exe_dir.join(if cfg!(windows) { "manager-cli.exe" } else { "manager-cli" });
        let _ = std::process::Command::new(&cli_path)
            .args(["download", "--driver", "-d"])
            .spawn();
    });

    // ── 日志 ──
    let weak3 = manager_window.as_weak();
    manager_window.on_refresh_logs(move || {
        if let Some(window) = weak3.upgrade() {
            window.set_log_content(read_log_tail().into());
            let lines = window.get_log_content().lines().count();
            window.set_log_line_count(lines as i32);
            window.set_log_total_count(lines as i32);
        }
    });

    let weak4 = manager_window.as_weak();
    manager_window.on_clear_logs(move || {
        if let Some(window) = weak4.upgrade() {
            window.set_log_content("".into());
            window.set_log_line_count(0);
        }
    });

    manager_window.run().expect("运行出错");
}

/// 从 config.toml 加载配置到窗口属性
fn load_config_to_window(window: &ManagerWindow) {
    let config_path = Path::new("config.toml");
    // 清除之前的保存校验错误
    window.set_config_save_error("".into());
    match ConfigFile::load_config(config_path) {
        Ok(config) => {
            window.set_config_toml_ok(true);

            window.set_config_login_username(config.login.info.username.clone().into());
            window.set_config_login_password(config.login.info.password.clone().into());
            window.set_config_login_service(config.login.info.service.clone().into());
            // 同步服务商 ComboBox 选中索引
            let svc_id = config.login.info.service.as_str();
            let services = SERVICES.lock().unwrap();
            if let Some(idx) = services.iter().position(|s| s.id == svc_id) {
                window.set_config_service_current_index(idx as i32);
            }
            drop(services);
            window.set_config_login_eportal(config.login.config.eportal.clone().into());
            window.set_config_login_timout(config.login.config.timout.to_string().into());
            window.set_config_query_interval(config.query.interval.to_string().into());

            // 填充连接检测条目
            let entries: Vec<ConnectEntry> = config.query.connect.iter().map(|c| {
                ConnectEntry { url: c.url.clone().into(), value: c.val.clone().into() }
            }).collect();
            let model = std::rc::Rc::new(slint::VecModel::from(entries));
            window.set_config_connect_entries(model.into());

            if let Some(ref chrome) = config.driver.chrome_config {
                window.set_config_driver_port(chrome.port.to_string().into());
                window.set_config_driver_path(chrome.driver_path.clone().into());
                window.set_config_browser_path(chrome.browser_path.clone().into());
            }

            if let Ok(text) = fs::read_to_string(config_path) {
                window.set_config_toml_text(text.into());
            }
        }
        Err(_) => {
            // 配置文件不存在 — 标记状态并填充默认值
            window.set_config_toml_ok(false);
            let defaults: Vec<ConnectEntry> = vec![
                ConnectEntry {
                    url: "http://www.msftncsi.com/ncsi.txt".into(),
                    value: "Microsoft NCSI".into(),
                },
            ];
            let model = std::rc::Rc::new(slint::VecModel::from(defaults));
            window.set_config_connect_entries(model.into());
        }
    }
}

/// 从 elements.toml 解析 [service] 条目并填充 ComboBox 模型
fn load_services_to_window(window: &ManagerWindow) {
    let elements_path = Path::new("elements.toml");
    let content = match fs::read_to_string(elements_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[elements] 读取失败: {}", e);
            // 回退到默认列表
            let defaults = vec!["_service_0".to_string(), "_service_1".to_string(),
                "_service_2".to_string(), "_service_3".to_string()];
            let names: Vec<slint::SharedString> =
                vec!["校园网".into(), "中国移动".into(), "中国电信".into(), "中国联通".into()];

            let mut services = SERVICES.lock().unwrap();
            *services = defaults.iter().enumerate()
                .map(|(i, id)| ServiceEntry { id: id.clone(), name: names[i].to_string() })
                .collect();
            drop(services);

            window.set_config_service_names(
                slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(names)))
            );
            return;
        }
    };

    let mut entries: Vec<ServiceEntry> = Vec::new();
    let mut current_id = String::new();
    let mut current_name = String::new();

    for line in content.lines() {
        let line = line.trim();
        if let Some(_rest) = line.strip_prefix("[[service]]") {
            // 保存上一个条目
            if !current_id.is_empty() {
                entries.push(ServiceEntry { id: current_id.clone(),
                    name: if current_name.is_empty() { current_id.clone() } else { current_name.clone() } });
            }
            current_id.clear();
            current_name.clear();
        } else if let Some(val) = line.strip_prefix("id") {
            current_id = val.split('=').nth(1)
                .map(|v| v.trim().trim_matches('"').to_string())
                .unwrap_or_default();
        } else if let Some(val) = line.strip_prefix("name") {
            current_name = val.split('=').nth(1)
                .map(|v| v.trim().trim_matches('"').to_string())
                .unwrap_or_default();
        }
    }
    // 最后一个条目
    if !current_id.is_empty() {
        let id = current_id.clone();
        entries.push(ServiceEntry { id: current_id,
            name: if current_name.is_empty() { id } else { current_name } });
    }

    if entries.is_empty() {
        return;
    }

    let names: Vec<slint::SharedString> = entries.iter().map(|s| s.name.as_str().into()).collect();

    {
        let mut svcs = SERVICES.lock().unwrap();
        *svcs = entries;
    }

    window.set_config_service_names(
        slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(names)))
    );
}

/// 从窗口表单字段重建 config.toml 并写入文件
fn save_config_from_window(window: &ManagerWindow) {
    // ── 校验必填字段 ──
    let username = window.get_config_login_username();
    let password = window.get_config_login_password();
    let eportal = window.get_config_login_eportal();
    let timout = window.get_config_login_timout();
    let interval = window.get_config_query_interval();
    let driver_port = window.get_config_driver_port();
    let driver_path = window.get_config_driver_path();
    let browser_path = window.get_config_browser_path();
    let entries = window.get_config_connect_entries();

    let mut errors: Vec<&str> = Vec::new();
    if username.trim().is_empty() { errors.push("用户名不能为空"); }
    if password.trim().is_empty() { errors.push("密码不能为空"); }
    if eportal.trim().is_empty() { errors.push("门户地址不能为空"); }
    if timout.trim().is_empty() { errors.push("超时不能为空"); }
    if interval.trim().is_empty() { errors.push("检测间隔不能为空"); }
    if driver_port.trim().is_empty() { errors.push("驱动端口不能为空"); }
    if driver_path.trim().is_empty() { errors.push("Driver 路径不能为空"); }
    if browser_path.trim().is_empty() { errors.push("浏览器路径不能为空"); }
    if entries.row_count() == 0 { errors.push("连接检测列表不能为空"); }

    // 校验每个连接条目的 url 和 value 不能为空
    if entries.row_count() > 0 {
        let mut has_empty_url = false;
        let mut has_empty_val = false;
        for i in 0..entries.row_count() {
            if let Some(entry) = entries.row_data(i) {
                if entry.url.trim().is_empty() { has_empty_url = true; }
                if entry.value.trim().is_empty() { has_empty_val = true; }
            }
        }
        if has_empty_url { errors.push("连接条目URL不能为空"); }
        if has_empty_val { errors.push("连接条目值不能为空"); }
    }

    if !errors.is_empty() {
        window.set_config_save_error(errors.join("；").into());
        return;
    }

    // 清除之前的错误
    window.set_config_save_error("".into());

    // 从 ComboBox 选中索引获取服务商 id
    let svc_idx = window.get_config_service_current_index() as usize;
    let service_id = {
        let services = SERVICES.lock().unwrap();
        services
            .get(svc_idx)
            .map(|s| s.id.clone())
            .unwrap_or_else(|| "_service_0".into())
    };

    // 从模型收集连接条目
    let connections: Vec<(String, String)> = (0..entries.row_count())
        .filter_map(|i| entries.row_data(i))
        .map(|e| (e.url.into(), e.value.into()))
        .collect();

    let toml_content = build_toml_from_form(
        username.as_str(),
        password.as_str(),
        service_id.as_str(),
        eportal.as_str(),
        timout.as_str(),
        interval.as_str(),
        &connections,
        driver_port.as_str(),
        driver_path.as_str(),
        browser_path.as_str(),
    );

    if let Err(e) = fs::write("config.toml", &toml_content) {
        eprintln!("[config] 写入失败: {}", e);
    } else {
        window.set_config_toml_text(toml_content.into());
        window.set_config_toml_ok(true);
    }
}

/// 检查 Chrome 浏览器和 ChromeDriver 文件是否实际存在
fn check_chrome_paths(config_path: &Path) -> (bool, String, bool, String) {
    let cwd = std::env::current_dir().unwrap_or_default();

    let (browser_path_str, driver_path_str) = match ConfigFile::load_config(config_path) {
        Ok(ref config) => match &config.driver.chrome_config {
            Some(c) => (c.browser_path.clone(), c.driver_path.clone()),
            None => {
                // 有 config 但未配置 Chrome 字段，提示用户
                return (false, "未配置 browser_path".into(), false, "未配置 driver_path".into());
            }
        },
        Err(_) => {
            // 没有 config，回退到 manager-cli download 后的默认解压路径
            let plat = platform::detect_platform();
            let suffix = platform::exe_suffix();
            let chrome_p = cwd.join(format!("chrome-{plat}")).join(format!("chrome{suffix}"));
            let driver_p = cwd.join(format!("chromedriver-{plat}")).join(format!("chromedriver{suffix}"));
            return (
                chrome_p.exists(),
                chrome_p.display().to_string(),
                driver_p.exists(),
                driver_p.display().to_string(),
            );
        }
    };

    let chrome_path = cwd.join(&browser_path_str);
    let driver_path = cwd.join(&driver_path_str);

    (
        chrome_path.exists(),
        chrome_path.display().to_string(),
        driver_path.exists(),
        driver_path.display().to_string(),
    )
}

/// 读取日志文件的末尾若干行
fn read_log_tail() -> String {
    let log_path = Path::new("neco.log");
    match std::fs::read_to_string(log_path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let start = if lines.len() > 200 { lines.len() - 200 } else { 0 };
            lines[start..].join("\n")
        }
        Err(_) => "无法读取日志文件".into(),
    }
}

/// 从表单字段重建完整 TOML 配置文本
fn build_toml_from_form(
    username: &str, password: &str, service: &str,
    eportal: &str, timout: &str,
    interval: &str,
    connections: &[(String, String)],
    driver_port: &str, driver_path: &str, browser_path: &str,
) -> String {
    let connect_entries: String = connections.iter()
        .map(|(url, val)| {
            format!("[[query.connect]]\nurl = \"{}\"\nvalue = \"{}\"", url, val)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        r#"[login.info]
username = "{}"
password = "{}"
service = "{}"

[login.config]
eportal = "{}"
timout = {}

[query]
interval = {}

{}

[driver]
driver_type = "Chrome"

[driver.chrome]
port = {}
driver_path = "{}"
browser_path = "{}"
"#,
        username, password, service,
        eportal, timout,
        interval,
        connect_entries,
        driver_port, driver_path, browser_path,
    )
}

fn format_uptime(elapsed_secs: u64) -> String {
    let h = elapsed_secs / 3600;
    let m = (elapsed_secs % 3600) / 60;
    let s = elapsed_secs % 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
