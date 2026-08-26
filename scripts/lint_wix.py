#!/usr/bin/env python3
"""Structural lint for wix/main.wxs.

Catches the classes of WiX errors we repeatedly hit on CI before they
reach Windows runners:
  - CNDL0205: root Directory must be TARGETDIR/SourceDir
  - CNDL0062: Component/@Directory under a Directory-bound ComponentGroup
  - LGHT0298: File Ids containing dots (bind variables split on '.')
  - unreferenced loose Components that would never install

Usage: python3 scripts/lint_wix.py [path-to-wxs]
"""
import sys
import xml.etree.ElementTree as ET

NS = {"w": "http://schemas.microsoft.com/wix/2006/wi"}


def main() -> int:
    path = sys.argv[1] if len(sys.argv) > 1 else "wix/main.wxs"
    root = ET.parse(path).getroot()

    # 1. Root directory check.
    for frag in root.findall(".//w:Fragment", NS):
        top = frag.find("w:Directory", NS)
        assert top is not None and top.get("Id") == "TARGETDIR" and top.get(
            "Name"
        ) == "SourceDir", "CNDL0205: root Directory must be TARGETDIR/SourceDir"

    # 2. ComponentGroup/@Directory + nested Component/@Directory conflict.
    for cg in root.findall(".//w:ComponentGroup", NS):
        gdir = cg.get("Directory")
        for c in cg.findall("w:Component", NS):
            assert not (gdir and c.get("Directory")), (
                f"CNDL0062: Component {c.get('Id')} sets Directory under a "
                "Directory-bound ComponentGroup"
            )

    # 3. Dotted File ids break bind variables.
    for f in root.findall(".//w:File", NS):
        fid = f.get("Id")
        assert "." not in fid, f"LGHT0298: File Id '{fid}' must not contain dots"

    # 4. Every loose Component must be referenced by the Feature.
    feat = root.find(".//w:Feature", NS)
    refs = {e.get("Id") for e in feat} | {
        e.get("Id") for e in feat.findall("w:ComponentGroupRef", NS)
    }
    group_comp_ids = {
        c.get("Id")
        for cg in root.findall(".//w:ComponentGroup", NS)
        for c in cg.findall("w:Component", NS)
    }
    for c in root.findall(".//w:Component", NS):
        cid = c.get("Id")
        if cid not in group_comp_ids:
            assert cid in refs, f"unreferenced component {cid} would never install"

    # 5. LGHT0091: Dialog ids are GLOBAL in the linked MSI. WiX v3 does
    # NOT merge a second Dialog with a library dialog's Id — it fails
    # with a duplicate symbol. Any dialog we declare must therefore use
    # an Id the referenced UI library does not already define
    # (WixUI_InstallDir brings its own InstallDirDlg, WelcomeDlg, ...).
    LIBRARY_DIALOGS = {
        "BrowseDlg", "CancelDlg", "CustomizeDlg", "DiskCostDlg", "ErrorDlg",
        "ExitDialog", "FatalError", "FilesInUse", "InstallDirDlg",
        "InvalidDirDlg", "LicenseAgreementDlg", "MaintenanceTypeDlg",
        "MaintenanceWelcomeDlg", "MsiRMFilesInUse", "OutOfDiskDlg",
        "OutOfRbDiskDlg", "PrepareDlg", "ProgressDlg", "ResumeDlg",
        "UserExit", "VerifyReadyDlg", "WaitForCostingDlg", "WelcomeDlg",
        "WixUIAdvertiseExitDialog", "AdvancedWelcomeEulaDlg", "WelcomeEulaDlg",
        "InstallScopeDlg", "InstallPerMachineDlg",
    }
    ours = [d.get("Id") for d in root.findall(".//w:Dialog", NS)]
    clash = sorted(set(ours) & LIBRARY_DIALOGS)
    assert not clash, (
        "LGHT0091: our Dialog Id(s) collide with the WixUI library: "
        + ", ".join(clash)
        + " — use a unique Id and reroute with Publish elements"
    )
    assert len(ours) == len(set(ours)), "duplicate Dialog Id in our source"

    # 6. CNDL0107: Dialog requires Width/Height even when it looks
    # "derived" from a library dialog (it never is — see rule 5).
    for d in root.findall(".//w:Dialog", NS):
        assert d.get("Width") and d.get("Height"), (
            f"CNDL0107: Dialog {d.get('Id')} misses Width/Height"
        )

    print(f"wix structural lint OK: {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
