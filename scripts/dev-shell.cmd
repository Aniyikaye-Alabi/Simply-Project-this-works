@echo off
rem Opens a shell with the VS 2022 Build Tools environment loaded.
rem Needed on this machine because the newer VS 2026 Community install is
rem missing lib\x64, and Rust picks the newest Visual Studio it finds.
rem Remove this once VS 2026 is repaired (Visual Studio Installer > Repair).
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul || exit /b 1
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
if "%~1"=="" (cmd /k) else (%*)
