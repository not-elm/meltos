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
meltos structs new <file_path> <line_no>

meltos structs speak <thread_id> <message>

meltos structs reply <thread_id> <message_no> <message>

# ブランチ所有者のみ
meltos structs close <thread_id>

# ディスカッション表示
meltos structs cat <thread_id>
```