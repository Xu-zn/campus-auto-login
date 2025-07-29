# 校园网自动登录

## 使用方法

1. 下载并解压
2. 将`chrome`和`chromedriver`放置到解压后的目录中
3. 运行`auto-login.exe`
4. 运行`auto-login-manager.exe`以停止`auto-login.exe`的运行

### 目录结构
```bash
AutoLogin
├─ auto-login-manager.exe
├─ auto-login.exe
├─ config.example.toml
├─ chromedriver.exe
└─ chrome-win64
   ├─ 其他文件...
   └─ chrome.exe
```

### Chrome
[官方下载地址](https://googlechromelabs.github.io/chrome-for-testing/) ，需要下载相同版本的`chrome`和`chrome-driver`

## 配置文件

配置文件为`config.toml`，需要与`auto-login.exe`在同一目录下
> 可以将`config.example.toml`重命名为`config.toml`

格式为TOML格式，具体格式如下：

```toml
connection_wait = 40 # 连接检查间隔时间(s)，不建议间隔太短

[login]
username = "..." # 学号
password = "..." # 密码
service = "" # 服务，填写校园网、南京移动、常州电信、常州联通，旧版填写_service_0, _service_1, _service_2, _service_3
wait_seconds = 3 # 浏览器打开网页后的等待时间(s)，防止页面元素加载不完全
# eportal = "http://eportal...." # 登陆校园网的url，旧版需要填写，新版限制只能是河海的登录地址


# 双层中括号表示列表，即多组连接检查地址和检查值，会遍历所有连接测试，直到有一个测试为连通就返回
[[connection]] 
# 连接检查地址，即程序会通过GET请求该地址，如果返回值与value相同，则认为网络已连接
url = "http://www.msftncsi.com/ncsi.txt"
# 连接检查值
value = "Microsoft NCSI" 

# 第二个连接测试地址
# [[connection]] 
# url = "http://www.msftncsi.com/ncsi.txt"
# value = "Microsoft NCSI" 

# 以下相关文件放在同级目录下
[webdriver]
# Chrome.exe所在目录
chrome_path = "chrome-win64"
# chrome driver的应用名
driver_path = "chromedriver.exe"

```

## Todo
- [x] 检测网络连接状态及自动登陆
- [x] 管理端GUI
  - [ ] 管理端操作auto-login的运行和退出
  - [ ] 管理端查看auto-login的运行状态
  - [ ] 管理端修改配置文件
  - [ ] 管理端设置开机自启的计划任务


## 注意事项

1. 程序通过chromedriver以headless模式运行chrome，模拟人的行为操作浏览器实现自动登陆
2. 由于检测网线或无线网是否接入时使用了Windows底层的Win32库，在win11上Windows Defender可能会报病毒
3. 不支持非win及win10以下的系统
4. 建议**插网线**
5. 开机自启需要在`任务计划程序`中添加任务