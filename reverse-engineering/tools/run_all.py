#!/usr/bin/env python3
"""One-command reverse-engineering pipeline for The CobbleBot Caper.

Input: the user's original Windows projector EXE.
Output: uncompressed SWF, structural/action reports, decoded bitmaps/sounds,
logic/level manifests, bitmap-fill links, and native collision masks.

This is development tooling only. None of it is required by the final Rust EXE.
"""
from pathlib import Path
import argparse, subprocess, sys, shutil

def run(*args):
    print('+',' '.join(map(str,args)),flush=True); subprocess.run([str(x) for x in args],check=True)

def main():
    ap=argparse.ArgumentParser();ap.add_argument('projector',type=Path);ap.add_argument('outdir',type=Path);ns=ap.parse_args()
    here=Path(__file__).resolve().parent; out=ns.outdir.resolve(); out.mkdir(parents=True,exist_ok=True)
    src=out/'source'; reports=out/'reports'; bitmaps=out/'bitmaps'; sounds=out/'sounds'; manifests=out/'manifests'; masks=out/'collision_masks'
    for d in (src,reports,bitmaps,sounds,manifests,masks): d.mkdir(parents=True,exist_ok=True)
    run(sys.executable,here/'extract_projector.py',ns.projector,src)
    swf=src/'cobblebot.swf'
    run(sys.executable,here/'swf_re.py',swf,reports)
    run(sys.executable,here/'extract_bitmaps.py',swf,bitmaps)
    run(sys.executable,here/'extract_sounds.py',swf,sounds)
    run(sys.executable,here/'generate_manifests.py',reports,manifests,'--bitmaps',bitmaps/'manifest.json','--sounds',sounds/'manifest.json')
    shutil.copy2(src/'source-fingerprints.json',manifests/'source-fingerprints.json')
    run(sys.executable,here/'extract_shape_fills.py',swf,manifests/'shape-bitmap-fills.json')
    run(sys.executable,here/'render_collision_masks.py',swf,reports/'symbols.json',manifests/'level-instances.json',masks)
    print('\nReverse-engineering pipeline completed successfully:',out)
if __name__=='__main__':main()
