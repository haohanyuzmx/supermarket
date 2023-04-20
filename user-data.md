```mermaid
sequenceDiagram
		actor u as user
    participant w as wallet
    note over u:with token
    u->>+w:钱包管理操作（充值提现）
    w->>-u:结果
    
```



```mermaid
sequenceDiagram
		actor u as user
    participant h as home
    note over u:with token
    u->>+h:添加/删除/改变 地址
    h->>-u:结果
```

 ```mermaid
 
 ```

