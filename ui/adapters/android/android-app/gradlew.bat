@rem Gradle startup script for Windows
@setlocal
@set DIRNAME=%~dp0
@set APP_BASE_NAME=%~n0
@set APP_HOME=%DIRNAME%
@set DEFAULT_JVM_OPTS="-Xmx64m" "-Xms64m"
@set CLASSPATH=%APP_HOME%\gradle\wrapper\gradle-wrapper.jar
@"%JAVA_HOME%\bin\java.exe" %DEFAULT_JVM_OPTS% -classpath "%CLASSPATH%" org.gradle.wrapper.GradleWrapperMain %*
@endlocal
