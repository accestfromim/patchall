# patchall
## 基本用法
usage: 
```
patchall executable_file /whatever/path/
```
会把executable_file的所有依赖的.so文件放到/whatever/path/dependencies下并且用patchelf修改依赖路径

支持选项 
```
--lpath /path/to/librarys
```
当你的所有动态链接库不在原来的位置，而是在一个新的目录的时候，用这个选项。会去这个选项的目录下找所有的动态链接库位置
这个选项是为了本程序使用过后挪动dependencies目录后再次使用，对于其它的情况不能正常使用这个选项

确保在运行本程序之前就有patchelf工具

## 测试
用curl做个测试
```
mkdir test
cd test
cp $(which curl) .
ldd ./curl
patchall ./curl .
ldd ./curl
./curl baidu.com
mv dependencies dependencies_new
./curl baidu.com # 应该会报错，找不到动态库
patchall ./curl . --lpath ./dependencies_new
./curl baidu.com
```