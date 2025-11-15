# Page snapshot

```yaml
- generic [ref=e3]:
  - banner [ref=e4]:
    - generic [ref=e5]:
      - generic [ref=e6]:
        - img [ref=e7]
        - heading "SIS Kernel" [level=1] [ref=e9]
      - generic [ref=e10]:
        - generic [ref=e11]:
          - generic [ref=e12]: "Daemon:"
          - generic [ref=e13]: Disconnected
        - button "Refresh Status" [ref=e14] [cursor=pointer]
        - generic [ref=e15]:
          - generic [ref=e16]: "QEMU:"
          - generic [ref=e17]: Idle
  - main [ref=e18]:
    - generic [ref=e20]:
      - img [ref=e21]
      - heading "Daemon Not Running" [level=2] [ref=e23]
      - paragraph [ref=e24]:
        - text: "The sisctl daemon is required to manage QEMU instances. Please start the daemon first:"
        - code [ref=e25]: pnpm daemon
      - button "Refresh Status" [ref=e26] [cursor=pointer]
  - contentinfo [ref=e27]:
    - generic [ref=e29]: "Lines: 0 | Events: 0"
```