# HHU校园网自动登录

## 使用方法

1. 下载并解压
2. 运行`auto-login.exe`
3. 右键`停止运行.bat`，以管理员身份运行

## 配置文件

配置文件为`config.toml`，需要与`auto-login.exe`在同一目录下

格式为TOML格式，具体格式如下：

```toml
connection_wait = 60 # 连接检查间隔时间

[login]
username = "..." # 学号
password = "..." # 密码
service = "_service_1" # 服务，0为校园网，1为南京移动，2为常州电信，3为常州联通
wait_seconds = 3 # 浏览器打开网页后的等待时间，防止页面元素加载不完全


# 双层中括号表示列表，即多组连接检查地址和检查值，会遍历所有连接测试，有一个测试为连通就返回
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
# Chrome目录名
chrome_path = "chrome-win64"
# chrome driver的应用名
driver_path = "chromedriver.exe"

# TODO: 给定一组无线网络候选列表，实现自动连接
# [[wlan]]
# ssid = "Hohai University"
# password = "" # 没有密码就直接空字符串

# [[wlan]]
# ssid = "wlan name"
# password = "wlan password"

```



## 注意事项

1. 程序通过chromedriver以headless模式运行chrome，模拟人的行为操作浏览器实现自动登陆

2. 由于检测网线或无线网是否接入时使用了Windows底层的Win32库，在win11上Windows Defender可能会报病毒

3. 不支持win10以下的系统

4. 