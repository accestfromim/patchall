# patchall
usage: patchall executable_file /whatever/path/
会把executable_file的所有依赖的.so文件放到/whatever/path/dependencies下并且用patchelf修改依赖路径

确保在运行本程序之前就有patchelf工具

现在还只是半成品，啥效果没有