@ECHO OFF
setlocal
SET parent=%~dp0
FOR %%a IN ("%parent%\..\..\..") DO SET "root=%%~fa"
@ECHO ON
docker run --rm -it --name="buzz-os" -v %root%:/buzz -w="/buzz" jvkdouk/buzz-os:latest
