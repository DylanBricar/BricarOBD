# -*- mode: python ; coding: utf-8 -*-
"""PyInstaller spec file for BricarOBD."""

import platform

block_cipher = None

a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[
        ('assets', 'assets'),
        ('data/dtc_descriptions.py', 'data'),
        ('data/__init__.py', 'data'),
        ('i18n.py', '.'),
        ('config.py', '.'),
    ],
    hiddenimports=[
        'customtkinter',
        'PIL',
        'serial',
        'serial.tools',
        'serial.tools.list_ports',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='BricarOBD',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=True if platform.system() == 'Darwin' else False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon='assets/icon.png',
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='BricarOBD',
)

# macOS .app bundle
if platform.system() == 'Darwin':
    app = BUNDLE(
        coll,
        name='BricarOBD.app',
        icon='assets/icon.png',
        bundle_identifier='com.dylanbricar.bricarobd',
        info_plist={
            'CFBundleName': 'BricarOBD',
            'CFBundleDisplayName': 'BricarOBD',
            'CFBundleVersion': '1.0.0',
            'CFBundleShortVersionString': '1.0.0',
            'NSHighResolutionCapable': True,
        },
    )
