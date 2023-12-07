### リポジトリ管理系のコマンド

```shell
meltos open
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
meltos thread new <file_path> <line_no>

meltos thread speak <thread_id> <message>

meltos thread reply <thread_id> <message_no> <message>

# ブランチ所有者のみ
meltos thread close <thread_id>

# ディスカッション表示
meltos thread cat <thread_id>
```