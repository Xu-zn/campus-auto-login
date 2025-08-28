# 校园网自动登录

## 使用方法

1. 下载并解压

2. 在目录下打开终端，运行`./manager-cli download`，下载`chrome`和`chrome-driver`

  > 对于windows on arm设备，使用`./manager-cli download --platform win64`下载

3. 运行`./manager-cli config --create`，创建配置文件

4. 修改配置文件中的`username`和`password`，以及`service`

5. windows下直接双击`auto-login.exe`运行，linux下执行`nohup ./auto-login >> auto-login.log 2>&1 &`

6. 停止运行, `./manager-cli stop`

## 配置文件

```toml
[login.info]
username = "" # 学号
password = "" # 密码
service = "南京移动" # 服务：校园网，南京移动、常州电信、常州联通

[login.config]
eportal = "http://eportal.hhu.edu.cn" # 登陆页面，不用改
timout = 3 # 等待页面超时时间，单位秒

[query]
interval = 15 # 检测间隔，单位秒，不建议太快，因为打开chrome很慢，容易导致同时打开两个chrome，同时执行登陆流程

[[query.connect]]
url = "https://captive.apple.com/hotspot-detect.html" # 用于检测网络联通性的网页
value = "Success" # 该网页返回的字符串

[[query.connect]]
url = "http://www.msftncsi.com/ncsi.txt"
value = "Microsoft NCSI"

[driver]
driver_type = "Chrome" # 不改，因为没实现Firefox的功能

[driver.chrome]
port = 18888 # chromedriver的端口，不建议小于10000的端口号
driver_path = "chromedriver-linux64/chromedriver" # chromedriver的路径，支持绝对路径以及本目录下的相对路径
browser_path = "chrome-linux64/chrome" # chrome的路径，支持绝对路径以及本目录下的相对路径
```

## Todo
- [x] 检测网络连接状态及自动登陆
- [x] 管理端GUI
  - [x] 管理端操作auto-login
  - [ ] 管理端修改配置文件
  - [ ] 管理端设置开机自启的计划任务
- [ ] 使用miniblink替代chrome

## 注意事项

1. 程序通过chromedriver以headless模式运行chrome，模拟人的行为操作浏览器实现自动登陆
2. 建议**插网线**
3. 不要使用高版本的chrome，会有问题，预设的版本号在`.chrome-version`中定义

## 开机自启设置
