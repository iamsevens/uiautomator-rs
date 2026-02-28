@echo off
REM UIAutomator Test App Build Script

echo Building UIAutomator Test App...

REM Clean previous build
call gradlew.bat clean

REM Build debug APK
call gradlew.bat assembleDebug

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build successful!
    echo.
    echo APK location:
    echo   app\build\outputs\apk\debug\app-debug.apk
    echo.
    echo To install:
    echo   adb install app\build\outputs\apk\debug\app-debug.apk
    echo.
) else (
    echo.
    echo Build failed!
    exit /b 1
)
