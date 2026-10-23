import os
import shutil
from pathlib import Path

BASE_DIR = Path("TG/data")
PHOTOS_DIR = BASE_DIR / "photos"
TEXTS_DIR = BASE_DIR / "text"
GROUPED_DIR = BASE_DIR / "grouped"

def group_files():
    print(f"Checking paths: {PHOTOS_DIR}, {TEXTS_DIR}")
    if not PHOTOS_DIR.exists() or not TEXTS_DIR.exists():
        print("Error: Data directories not found.")
        return

    GROUPED_DIR.mkdir(exist_ok=True)

    chats = set()
    if PHOTOS_DIR.exists():
        dirs = [d.name for d in PHOTOS_DIR.iterdir() if d.is_dir()]
        print(f"Found chat dirs in photos: {dirs}")
        chats.update(dirs)
    if TEXTS_DIR.exists():
        dirs = [d.name for d in TEXTS_DIR.iterdir() if d.is_dir()]
        print(f"Found chat dirs in texts: {dirs}")
        chats.update(dirs)

    for chat_id in chats:
        print(f"Processing chat {chat_id}...")
        chat_photos = PHOTOS_DIR / chat_id
        if chat_photos.is_dir():
            files = list(chat_photos.iterdir())
            print(f"  Found {len(files)} photos")
            for file_path in files:
                if file_path.is_file():
                    filename = file_path.name
                    parts = filename.split('_')
                    if len(parts) >= 2:
                        msg_id = parts[1].split('.')[0]
                        target_folder = GROUPED_DIR / chat_id / msg_id
                        target_folder.mkdir(parents=True, exist_ok=True)
                        shutil.copy2(file_path, target_folder / filename)
                        print(f"    Copied {filename} to {msg_id}")

        chat_texts = TEXTS_DIR / chat_id
        if chat_texts.is_dir():
            files = list(chat_texts.iterdir())
            print(f"  Found {len(files)} texts")
            for file_path in files:
                if file_path.is_file():
                    filename = file_path.name
                    parts = filename.split('_')
                    if len(parts) >= 2:
                        msg_id = parts[1].split('.')[0]
                        target_folder = GROUPED_DIR / chat_id / msg_id
                        target_folder.mkdir(parents=True, exist_ok=True)
                        shutil.copy2(file_path, target_folder / filename)
                        print(f"    Copied {filename} to {msg_id}")

    print("Grouping complete!")

if __name__ == "__main__":
    group_files()
