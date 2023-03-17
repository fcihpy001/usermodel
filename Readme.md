
创建镜像文件
docker-compse up -d

<!-- 查看镜像进程状态 -->
docker ps

<!-- 连接数据 -->
psql -h 127.0.0.1 -p 5432 -U actix actix

\d \q clear

<!-- 执行sql脚本 -->
psql -h 127.0.0.1 -p 5432 -U actix actix < database.sql