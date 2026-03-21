# 015_failure_after_partial_success_leaves_changes

Tests that when a patch has multiple hunks and a later hunk fails, earlier successful changes are still persisted to the filesystem (no rollback).
