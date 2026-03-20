@echo off
REM Build BricarOBD.exe for Windows
echo === BricarOBD Windows Build ===

REM Check PyInstaller
python -c "import PyInstaller" 2>nul || (
    echo Installing PyInstaller...
    pip install pyinstaller
)

REM Clean previous builds
if exist build rmdir /s /q build
if exist dist rmdir /s /q dist

REM Build .exe
echo Building .exe...
pyinstaller build.spec --noconfirm

echo.
echo === Build Complete ===
echo   .exe: dist\BricarOBD\BricarOBD.exe
echo.
echo To distribute: zip the dist\BricarOBD folder
pause
