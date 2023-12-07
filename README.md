## Commands


### リポジトリ管理系のコマンド
```shell
meltos join <room_id>
meltos branch <local_branch> --from <remote_branch>
meltos checkout <local_branch>
meltos commit -M <message>
meltos push
meltos merge <remote_branch>
```

### ディスカッションコマンド
```shell
# ディスカッションIDが返される
meltos discussion new <file_path> <line_no>

meltos discussion speak <discussion_id> <message>

# ブランチ所有者のみ
meltos discussion close <discussion_id>

# ディスカッション表示
meltos discussion view <discussion_id>
```