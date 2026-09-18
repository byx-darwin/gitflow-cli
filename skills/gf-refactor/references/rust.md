# Rust Refactoring Layer

**Detection:** `Cargo.toml` at project root (per `gf-quality/references/detector.md`).

## 校验命令

**每个手法应用后各自单独运行，不得合并成一次跑到底再回头看。**

```bash
make check   # 编译检查（不生成代码，比 build 快）
make test    # nextest 全量测试
```

若 `make check` 失败 → 撤销本次编辑，不进入测试。若 `make check` 通过但
`make test` 失败 → 撤销本次编辑，不应用下一个手法。

引入新依赖或改动 `Cargo.toml` 的手法（例如 Replace Constructor with Factory
Function 用到 `typed-builder`）额外跑：

```bash
make lint    # fmt + clippy，捕获新代码是否符合本仓库 pedantic 门槛
```

**不得使用 `cargo clean`**——CLAUDE.md 明确禁止，需要时须先问用户。

## 惯用法映射

| 手法 | Rust 落点 |
|---|---|
| Extract Function 提炼函数 | 私有 `fn`，签名从调用点的类型推导 |
| Introduce Parameter Object 引入参数对象 | 字段 >5 时用 `typed-builder`（本仓库约定），否则普通 struct |
| Replace Conditional with Polymorphism 以多态取代条件表达式 | `enum` + `match`（穷尽性检查代替「遗漏分支」的语义风险）；跨 crate 扩展点才用 trait object |
| Replace Type Code with Subclasses 以子类取代类型码 | `enum` variants，而非类层次（Rust 无类继承） |
| Extract Superclass / Collapse Hierarchy 提炼超类/折叠继承体系 | `trait` 默认方法，或直接合并（组合优先于继承） |
| Encapsulate Variable 封装变量 | 私有字段 + `pub fn` getter |
| Change Value to Reference 将值对象改为引用对象 | `Rc<RefCell<T>>` 或 `Arc<Mutex<T>>`——见陷阱 1 |
| Replace Derived Variable with Query 以查询取代派生变量 | 移除缓存字段，改成方法；若原字段是缓存，见陷阱 2 |

## 语义陷阱判例

### 陷阱 1: Change Value to Reference 与 `Drop` 时机

把一个值类型字段改成 `Rc<RefCell<T>>` 后，原本随值类型的所有者一起在作用域
结束时确定性析构的 `T`，析构时机变成"最后一个 `Rc` 引用计数归零时"——若 `T`
的 `Drop` 有可观察副作用（关闭文件、释放锁、写日志），析构时机从"编译期确定"
变成"运行期依赖引用计数"。这正是「可能变更」标注的具体体现，不能当等价手法
处理。

### 陷阱 2: Split Loop / Replace Loop with Pipeline 与迭代器惰性求值

Rust 的 `Iterator` 是惰性的：`.map(f)` 不会立即调用 `f`，只有被 `.collect()`
/ `.for_each()` / `for` 消费时才真正求值。把一个带副作用的 `for` 循环体拆成
`.iter().map(|x| { side_effect(x); x })` 而不消费返回值，`side_effect`
根本不会执行——这不是"重构后行为不变"，是重构后代码变成死代码。Split Loop /
Replace Loop with Pipeline 应用前必须确认：拆分出的每个迭代器链都有终结消费
者，且消费顺序与原循环的副作用顺序一致。

### 陷阱 3: Extract Function 与闭包捕获

把一段引用外层可变局部变量的代码提炼成闭包（而非独立 `fn`）时，闭包会捕获该
变量的可变借用，其生命周期从"和原作用域一致"变成"和闭包一致"——若闭包被存进
一个比原作用域更长寿的结构（如注册进事件回调表），编译器会报借用检查错误；若
通过 `move` 强行让闭包拥有变量所有权则消除了错误，但也悄悄改变了"谁最终拥有
这份状态"，需按变量是否实现 `Copy` 分两种情况判断：若该变量实现 `Copy`，
`move` 对闭包内部是隐式拷贝，外层变量本身不受影响，仍持有移动前的值——问题
不是"读到影子"，而是"闭包内外两份独立拷贝，调用方后续的修改不会反映到闭包
里，反之亦然"，这才是真正的语义分歧；若该变量不实现 `Copy`，闭包获得所有权
后，调用方对该变量的任何后续读取是**编译期错误**（E0382，use-after-move），
不存在"读到影子"这回事——问题在于原本"提炼后调用方还能继续用这个变量"的假设
在提炼成 `move` 闭包后不再成立，需要走 borrow checker 报错而不是静默产生
错误结果。Extract Function 标注为「等价」隐含的前提是"提炼成独立函数，不提炼
成捕获式闭包"，一旦落到闭包就需要重新评估为 条件等价。

### 陷阱 4: Remove Setting Method 与测试 fixture 复用路径

若某字段的设值方法（setter）被移除，字段变为仅构造时可设；但该类型若派生了
`#[derive(Default)]` 且其它代码通过 `T::default()` 再手动赋值来"复用" setter
路径（常见于测试 fixture），移除 setter 会使这条路径编译失败——这不是行为
改变，是接口改变，但由于错误只在依赖它的下游代码触发，本地 `make check` 未必
覆盖，须额外搜索 workspace 内全部调用点。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `cargo-nextest` 未安装（`make test` 依赖它） | 提示安装；本层的校验命令强制依赖它，不得回退到裸 `cargo test`（覆盖率/汇报格式不一致） |
| Rust 工具链版本与 `rust-toolchain.toml` 不符 | 停止，不自行切换工具链版本（配置文件变更需用户确认） |
