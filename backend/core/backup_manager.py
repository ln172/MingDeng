"""
Backup Manager Module
Handles data backup and restoration
"""

import json
import shutil
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Optional
from .paths import get_data_dir


class BackupManager:
    """Data backup and restoration manager"""

    MAX_BACKUPS = 10  # Maximum number of backups to keep

    def __init__(self, data_dir: Optional[str] = None, backup_dir: Optional[str] = None):
        base_dir = Path(data_dir) if data_dir else get_data_dir()
        self.data_dir = base_dir
        self.backup_dir = Path(backup_dir) if backup_dir else base_dir / "backups"
        self.backup_dir.mkdir(parents=True, exist_ok=True)

    def create_backup(self, reason: str = "Manual backup") -> Dict[str, Any]:
        """
        Create a backup of current data

        Args:
            reason: Reason for backup

        Returns:
            Backup info
        """
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_id = f"backup_{timestamp}"
        backup_path = self.backup_dir / backup_id

        try:
            # Create backup directory
            backup_path.mkdir(exist_ok=True)

            # Files to backup
            files_to_backup = [
                "todos.json",
                "library.json",
                "config.json"
            ]

            backed_up_files = []
            for filename in files_to_backup:
                source = self.data_dir / filename
                if source.exists():
                    dest = backup_path / filename
                    shutil.copy2(source, dest)
                    backed_up_files.append(filename)

            # Save backup metadata
            metadata = {
                "id": backup_id,
                "timestamp": timestamp,
                "datetime": datetime.now().isoformat(),
                "reason": reason,
                "files": backed_up_files
            }

            metadata_path = backup_path / "metadata.json"
            with open(metadata_path, 'w', encoding='utf-8') as f:
                json.dump(metadata, f, indent=2, ensure_ascii=False)

            # Clean up old backups
            self._cleanup_old_backups()

            return {
                "success": True,
                "backup": metadata
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }

    def _cleanup_old_backups(self):
        """Remove old backups if exceeding MAX_BACKUPS"""
        backups = self.list_backups()
        if len(backups) > self.MAX_BACKUPS:
            # Sort by timestamp (oldest first)
            backups.sort(key=lambda x: x["timestamp"])

            # Remove oldest backups
            to_remove = len(backups) - self.MAX_BACKUPS
            for i in range(to_remove):
                backup_id = backups[i]["id"]
                backup_path = self.backup_dir / backup_id
                if backup_path.exists():
                    shutil.rmtree(backup_path)

    def list_backups(self) -> List[Dict[str, Any]]:
        """
        List all available backups

        Returns:
            List of backup metadata
        """
        backups = []

        if not self.backup_dir.exists():
            return backups

        for backup_path in self.backup_dir.iterdir():
            if backup_path.is_dir():
                metadata_path = backup_path / "metadata.json"
                if metadata_path.exists():
                    try:
                        with open(metadata_path, 'r', encoding='utf-8') as f:
                            metadata = json.load(f)
                            backups.append(metadata)
                    except Exception as e:
                        print(f"Error reading backup metadata: {e}")

        # Sort by timestamp (newest first)
        backups.sort(key=lambda x: x["timestamp"], reverse=True)
        return backups

    def _validate_backup_id(self, backup_id: str) -> bool:
        """Reject backup IDs that could cause path traversal"""
        return bool(backup_id) and "/" not in backup_id and "\\" not in backup_id and ".." not in backup_id

    def restore_backup(self, backup_id: str) -> Dict[str, Any]:
        """
        Restore data from a backup

        Args:
            backup_id: Backup ID to restore

        Returns:
            Restoration result
        """
        if not self._validate_backup_id(backup_id):
            return {"success": False, "error": "非法 backup_id"}

        backup_path = self.backup_dir / backup_id

        if not backup_path.exists():
            return {
                "success": False,
                "error": "备份不存在"
            }

        try:
            # Read backup metadata
            metadata_path = backup_path / "metadata.json"
            with open(metadata_path, 'r', encoding='utf-8') as f:
                metadata = json.load(f)

            # Create a backup of current state before restoring
            self.create_backup(reason=f"Before restoring {backup_id}")

            # Restore files
            restored_files = []
            for filename in metadata.get("files", []):
                source = backup_path / filename
                dest = self.data_dir / filename
                if source.exists():
                    shutil.copy2(source, dest)
                    restored_files.append(filename)

            return {
                "success": True,
                "restored_files": restored_files,
                "backup": metadata
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }

    def delete_backup(self, backup_id: str) -> bool:
        """
        Delete a specific backup

        Args:
            backup_id: Backup ID to delete

        Returns:
            Success status
        """
        if not self._validate_backup_id(backup_id):
            return False

        backup_path = self.backup_dir / backup_id

        if not backup_path.exists():
            return False

        try:
            shutil.rmtree(backup_path)
            return True
        except Exception as e:
            print(f"Error deleting backup: {e}")
            return False


# Global backup manager instance
backup_manager = BackupManager()
