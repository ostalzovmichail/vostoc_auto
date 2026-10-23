import os
import shutil
from pathlib import Path

BASE_DIR = Path("TG/data/grouped")

def rename_folders():
    if not BASE_DIR.exists():
        print("Error: Grouped directory not found.")
        return

    # Iterate through each chat folder
    for chat_dir in BASE_DIR.iterdir():
        if not chat_dir.is_dir():
            continue
        
        print(f"Processing chat: {chat_dir.name}")
        
        # Iterate through message folders (msg_id)
        # List them first to avoid issues while renaming
        msg_folders = [d for d in chat_dir.iterdir() if d.is_dir()]
        
        for folder in msg_folders:
            # Find the .txt file in this folder
            txt_files = list(folder.glob("*.txt"))
            if not txt_files:
                continue
            
            # Read the first line of the first text file found
            txt_file = txt_files[0]
            try:
                with open(txt_file, 'r', encoding='utf-8') as f:
                    first_line = f.readline().strip()
                
                if not first_line:
                    continue
                
                # Sanitize first_line to be a valid folder name
                # Replace spaces with _ and remove invalid chars
                safe_name = first_line.replace(" ", "_").replace("/", "-").replace("\\", "-")
                # Remove any other characters that might be problematic
                safe_name = "".join(c for c in safe_name if c.isalnum() or c in ("_", "-", ".", " "))
                safe_name = safe_name.replace(" ", "_")
                
                new_folder_path = chat_dir / safe_name
                
                # Handle collisions (if two messages have the same first line)
                counter = 1
                original_safe_name = safe_name
                while new_folder_path.exists():
                    new_folder_path = chat_dir / f"{original_safe_name}_{counter}"
                    counter += 1
                
                print(f"  Renaming {folder.name} -> {new_folder_path.name}")
                folder.rename(new_folder_path)
                
            except Exception as e:
                print(f"  Error processing {folder.name}: {e}")

    print("Renaming complete!")

if __name__ == "__main__":
    rename_folders()
