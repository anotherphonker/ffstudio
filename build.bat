@echo off
setlocal EnableExtensions
title FF Studio - build
rem ============================================================
rem  FF Studio build scripti
rem  - cargo build --release calistirir
rem  - ffmpeg.zip bu klasorde varsa binary icine gomulur (all-in-one)
rem  Kullanim: build.bat            = sadece build
rem           build.bat run         = build + calistir
rem ============================================================

set "BUILD_ONLY=1"
if /i not "%~1"=="run" set "BUILD_ONLY=0"

echo.
echo  FF Studio build
echo  =================

where cargo >nul 2>nul
if errorlevel 1 goto :nocargo
cargo --version

if exist "%~dp0ffmpeg.zip" (
    echo  [ok] ffmpeg.zip bulundu - all-in-one build yapilacak.
) else (
    echo  [uyari] ffmpeg.zip yok - binary, sistem ffmpeg kullanacak.
    echo         All-in-one icin gyan.dev essentials zip'ini bu klasore
    echo         ffmpeg.zip adiyla koyup tekrar calistir.
)
echo.

cargo build --release
set "rc=%errorlevel%"
if not "%rc%"=="0" (
    echo.
    echo  [HATA] Build basarisiz oldu. Yukaridaki hata mesajlarina bak.
    pause
    exit /b %rc%
)

echo.
echo  ================= Bitti! =================
echo  Exe: %~dp0target\release\ffstudio.exe
echo.

if "%BUILD_ONLY%"=="1" goto :end
set /p "runit=  calistiralim mi?  e veya h: "
if /i not "%runit%"=="e" goto :end
start "" "%~dp0target\release\ffstudio.exe"

:end
pause
exit /b 0

:nocargo
echo.
echo  [HATA] cargo bulunamadi!
echo  Rust'u kur: https://rustup.rs  - varsayilan "stable MSVC" secenegini kullan.
echo  Kurulumdan sonra bu pencereyi kapatip build.bat'i tekrar calistir.
echo.
pause
exit /b 1
