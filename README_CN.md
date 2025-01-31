# ATTPS (AgentText Transfer Protocol Secure) on Solana

本文档将首先介绍 Solana 合约的基本结构，然后依次展开各个模块的实现，并在其中解释和 Solidity 版本合约的不同之处。

## Solana 合约架构

和 EVM 不同，Solana 是一个 Repo 对应一整个合约，而不像 Solidity 一样一个 contract 关键字对应一个合约。因此，我们把 ATTPS 中解耦的多个模块都放在了同一个合约中，并在代码层面将其解耦。ATTPS 的模块包括：ATTPS Factory 用于创建新的 Agent、ATTPS Manager 用于管理员接受 Agent 及 Agent Settings、ATTPS Proxy 是 Agent 调用的入口。

Solana 合约的文件结构如下。

```log
├── Cargo.lock           # Rust 依赖锁文件
├── Cargo.toml           # 项目(主包)依赖和配置
├── README.md            # 项目文档
├── script
│   ├── Cargo.toml       # Script(子包)的依赖和配置
│   ├── deploy.sh        # 合约部署脚本
│   └── interact.rs      # 合约交互脚本
└── src
    ├── constants.rs     # 系统常量
    ├── entrypoint.rs    # 合约的入口
    ├── error.rs         # 自定义错误代码
    ├── instruction.rs   # 合约的 instructions，可以理解为 interface
    ├── lib.rs           # 模块声明和导出
    ├── processor.rs     # 各个 instructions 的处理逻辑
    ├── state.rs         # 合约状态以及各个存储数据结构
    ├── test
    │   ├── instruction_test.rs     # 各个 instructions 的测试
    │   ├── state_test.rs           # 数据结构层面的测试
    │   ├── utils_agent_test.rs     # agent utils 相关函数的测试，比如 agent verify
    │   └── utils_manager_test.rs   # agent manager 相关函数的测试，比如 accept agent
    └── utils.rs         # 各种 utils 函数，比如创建 PDA 账户
```

我们依次解释各个代码文件的含义。

## Utils & State

Utils 中包含 3 段代码，分别是 DataAccountUtils、AgentUtils 和 AgentManagerUtils。而 State 中包含定义的各种数据结构。

DataAccountUtils 用于处理 Solana 的账户逻辑。由于 Solana 每次存储数据都需要使用一个 PDA（Program Derived Account），同时还需要在创建时指定 Account 的大小（存储空间越大、需要支付的 SOL 租金越多），因此 DataAccountUtils 中提供了创建 PDA 账户、序列化数据、验证账户所有权等函数。这里详细解释一下数据的存储方法。

1. PDA 存储结构
  
以 ContractInfo 这个数据结构举例， 它是我们定义的存储合约 Metadata 的数据结构。我们首先需要创建一个 PDA，然后把如下的 ContractInfo 数据结构序列化为一个 Vec<u8> 数组，最后把这个数组存进 PDA 中。ContractInfo 包含的前两个字段是定长的，包括合约 owner（Pubkey 是 32 bytes）、以及创建的 agent 总数 agent_counter（u128 是 16 bytes）；而后两个字段 type_and_version 和 agent_version 是可变长度的 String，但它们是也是可序列化的（先用 4 bytes 记录长度信息再用后面的 bytes 记录实际内容）。

    ```rust
    pub struct ContractInfo {
        pub owner: Pubkey,
        pub agent_counter: u128,
        pub type_and_version: String,
        pub agent_version: String,
    }
    ```

举个例子，如下的 ContractInfo 内容：
    ```rust
    ContractInfo { owner: 98zXLj446YJUVX37ENSRoPBchsWGEEUzd91q8L9gyMoe, agent_counter: 6, type_and_version: "AI Agent 1.0.0", agent_version: "AI Agent 1.0.0" }
    ```

可以被序列化为：
    ```rust
    // Vector view
    [84, 0, 0, 0, 120, 233, 154, 71, 177, 176, 116, 170, 161, 176, 49, 99, 94, 190, 93, 124, 192, 169, 158, 217, 54, 31, 179, 197, 47, 163, 136, 158, 243, 255, 116, 209, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0, 65, 73, 32, 65, 103, 101, 110, 116, 32, 49, 46, 48, 46, 48, 14, 0, 0, 0, 65, 73, 32, 65, 103, 101, 110, 116, 32, 49, 46, 48, 46, 48]
    // Hex string view
    0x5400000078e99a47b1b074aaa1b031635ebe5d7cc0a99ed9361fb3c52fa3889ef3ff74d1060000000000000000000000000000000e0000004149204167656e7420312e302e300e0000004149204167656e7420312e302e30
    ```

可以看出，前面的 0x54000000 = 84 是 ContractInfo 的整体长度（Little-Endian encoding，从右到左编码 uint），0x78e99a47b1b074aaa1b031635ebe5d7cc0a99ed9361fb3c52fa3889ef3ff74d1 是 owner Pubkey，0x06000000000000000000000000000000 = 6 是 agent_counter（Little-Endian encoding，u128 共 16 bytes），0x0e000000 = 14 是 type_and_version 的长度，0x4149204167656e7420312e302e30 是 "AI Agent 1.0.0" 的 utf-8 编码，后面的 agent_version 同理。

2. AgentInfo 的设计

Solidity 中可以自然的使用 s_agentSettings、s_agentSourceIds、s_allowedAgents 等 mapping 数据结构，但在 Solana 里是完全不行的。因此，我们设计了 AgentInfo 这个数据结构，用于存储所有的相关信息；同时，它会有一个额外的 agent_id 字段，它是自动递增的。而存储 AgentInfo 的 PDA 账户地址，则由合约计算得到（和 agent_id 有关），在链下也可以提前计算得到，也就是说，每个 Agent 的 PDA 地址都是预先确定好的

    ```rust
    pub struct AgentInfo {
        pub agent_id: u128,
        pub is_registered: bool,
        pub is_allowed: bool,
        pub is_removed: bool,
        pub agent_settings: AgentSettings,
        pub pending_settings: Option<AgentSettings>,
        pub agent_config: AgentConfig,
    }
    ```

字段解释如下：

- agent_id：agent 的唯一标识符，自动递增
- is_registered：agent 是否已注册，对应 Solidity 中的 s_registeringAgents
- is_allowed：agent 是否已被管理员允许，对应 Solidity 中的 s_allowedAgents
- is_removed：agent 是否已被管理员移除，在 Solidity 中无对应
- agent_settings：agent 的设置，对应 Solidity 中的 s_agentSettings
- pending_settings：agent 的 pending 设置，在 Solidity 中无对应，在下文中详细解释
- agent_config：agent 的配置，对应 Solidity 中的 s_agentConfigStates

这些字段都是可序列化的，因此可以被存储在 PDA 中。我们目前将总长度上限设定为了 4096 bytes。具体而言，一个 AgentInfo 的具体长度为：

    ```rust
    /// Total size in bytes = 38 + <sum of all string lengths>, where:
    /// - Each string field adds (4 + content length) bytes
    /// - Fixed fields add (8 + 1 + 1 + 8 = 18) bytes (timestamps, enums, etc)
    pub struct AgentHeader {
        pub version: String,
        pub message_id: String,
        pub source_agent_id: String,
        pub source_agent_name: String,
        pub target_agent_id: String,
        pub timestamp: u64,
        pub message_type: MessageType,
        pub priority: Priority,
        pub ttl: u64,
    }

    /// - threshold: 1 byte (u8)
    /// - converter_address: 32 bytes (Pubkey)
    /// - signers: (4 + 20*signers.len()) bytes (Vec<[u8; 20]> for Ethereum addresses)
    /// - agent_header: See AgentHeader struct size calculation
    ///
    /// Total size = 37 + (20*signers.len()) + agent_header_size
    pub struct AgentSettings {
        pub signers: Vec<[u8; 20]>,
        pub threshold: u8,
        pub converter_address: Pubkey,
        pub agent_header: AgentHeader,
    }

    /// - config_digest: 32 bytes ([u8; 32])
    /// - config_block_number: 8 bytes (u64)
    /// - is_active: 1 byte (bool)
    /// - settings: See AgentSettings struct size calculation
    ///
    /// Total size = 41 + settings_size
    pub struct AgentConfig {
        pub config_digest: [u8; 32],
        pub config_block_number: u64,
        pub is_active: bool,
        pub settings: AgentSettings,
    }

    /// - agent_id: 16 bytes (u128)
    /// - is_registered: 1 byte (bool)
    /// - is_allowed: 1 byte (bool)
    /// - is_removed: 1 byte (bool)
    /// - is_new_settings: 1 byte (bool)
    /// - agent_settings: See AgentSettings struct size calculation
    /// - pending_settings: See AgentSettings struct size calculation
    /// - agent_config: See AgentConfig struct size calculation
    ///
    /// Total size = 20 + 2 * settings_size + config_size
    ///   ~ 20 + 2 * (37 + (20*signers.len()) + agent_header_size)
    ///        + (41 + 37 + (20*signers.len()) + agent_header_size)
    ///   ~ 172 + 60 * signers.len() + 3 * (38 + strings.len())
    ///   ~ 286 + 60 * signers.len() + 3 * strings.len()
    pub struct AgentInfo {
        pub agent_id: u128, // Unique auto-incrementing identifier
        pub is_registered: bool,
        pub is_allowed: bool,
        pub is_removed: bool,
        pub agent_settings: AgentSettings,
        pub pending_settings: Option<AgentSettings>,
        pub agent_config: AgentConfig,
    }
    ```

我们假设有 20 个 signers，而各个 String 的总长度为 300，那么总共的 bytes 大约为 2400。因此，我们将上限设为 4096 是可以满足大部分需求的。

3. DataAccountUtils

除了创建 PDA 用于存储上述各个数据结构以外，DataAccountUtils 还提供了读取和写入 PDA 数据、以及检验 owner 的方法，函数签名分别如下：

    ```rust
    pub fn write_account_data<Data: BorshSerialize>(data_account: &AccountInfo, content: Data) -> ProgramResult;
    pub fn read_account_data<Data: BorshDeserialize>(data_account: &AccountInfo) -> Result<Data, ProgramError>;
    pub fn check_account_ownership(program_id: &Pubkey, account: &AccountInfo) -> ProgramResult;
    ```

就检验 owner 而言，这里的 owner 是「Data Account 层面」的，而不是「合约层面」的。也就是说，由于是本合约创建了某个 PDA，所以之后也拥有更改该 PDA 内部数据的权限（也就是 owner 权限），而其他任何 EOA 或合约都是没有更改这部分数据的权限的，包括合约本身的管理者 owner。

4. AgentUtils

这部分代码主要重写了 `Agent.sol` 中的各个方法，包括 `address_exists`、`verify_signature`、`verify_zk`、`verify_merkle`（后两者暂未实现）。需要注意的是，由于 signers 仍然是 EVM 地址而不是 Solana 地址，因此其类型是 `[u8; 20]` 而不是 `Pubkey`。此外，由于 Solidity 的 `ecrecover` 函数可以直接恢复 EVM 地址，而普通的 Secp256k1 签名恢复的是 Secp256k1 公钥，因此需要额外使用 `pubkey_to_eth_address` 函数将公钥转换为 EVM 地址。

5. AgentManagerUtils

这部分代码主要重写了 `AgentManager.sol` 中的各个方法，包括 `is_valid_uuid`、`validate_agent_header`、`setting_digest_from_settings_data` 等方法。注意 `setting_digest_from_settings_data` 函数会和 Solidity 版本的有略微不同，但整体逻辑是相同的，而且由于该函数只需要保证同链内的一致性，因此这个差异是允许的。

---

## Instructions

这里的 instructions 可以理解为“所有外部能够调用的函数的集合”，作用和 Solidity 中的 ABI 类似。Solana 中每次调用合约时，除了给出 `program_id`（也就是合约地址 Pubkey）以外，还要给出 `accounts` 和 `data` 两个参数，其中 `data` 包含了指定要调用的函数以及对应的参数，而 `accounts` 则包含了在这次调用中所涉及到的所有账户的信息。后者相对于 Solidity 而言是一个完全的新的概念。

我们在 `instruction.rs` 中定义了这些入口，并在注释中给出了需要传入的 accounts。包括：

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum AgentInstruction {
    /// [0] Initialize the contract
    ///
    /// 0. [signer] payer
    /// 1. [] owner
    /// 2. [writable] contract_info
    Initialize,

    /// [1] Create a new agent (create a new data account)
    ///
    /// 0. [signer] payer
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    CreateAgent,

    /// [2] Register an agent (write to agent data account)
    ///
    /// 0. [writable] contract_info
    /// 1. [writable] agent data account
    RegisterAgent { agent_settings: AgentSettings },

    /// [3] Create & register an agent ([1] + [2])
    ///
    /// 0. [signer] payer
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    CreateAndRegisterAgent { agent_settings: AgentSettings },

    /// [4] Accept an agent
    ///
    /// 0. [signer] owner
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    AcceptAgent,

    /// [5] Change an agent setting proposal
    ///
    /// 0. [writable] agent data account
    ChangeAgentSettingProposal { proposed_settings: AgentSettings },

    /// [6] Accept an agent setting proposal
    ///
    /// 0. [signer] owner
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    AcceptAgentSettingProposal,

    /// [7] Remove an agent
    ///
    /// 0. [signer] owner
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    RemoveAgent,

    /// [8] Verify a message of an agent
    ///
    /// 0. [] agent data account
    Verify {
        settings_digest: [u8; 32],
        payload: MessagePayload,
    },
}
```

一共 Initialize、CreateAgent、RegisterAgent、CreateAndRegisterAgent、AcceptAgent、ChangeAgentSettingProposal、AcceptAgentSettingProposal、RemoveAgent、Verify 这 9 个函数，注释中标注的 accounts 我们再来做进一步解释：

- `[signer]` 字样代表该 account 需要对这笔交易进行签名。比如在 `AcceptAgent` 中，`owner` account 需要对这笔交易签名，这笔交易才能执行。也就意味着 `owner` 才拥有接受某个 Agent 的权限。
- `[writable]` 字样代表该 account 需要被写入数据。比如在 `CreateAgent` 中，`contract_info` account 和 `agent_data` account 都需要被写入（修改）数据，这笔交易才能执行。
- `contract_info` 是存储上述 ContractInfo 数据结构的位置，该合约只有一个 `contract_info` account。
- `agent_data` 是存储上述 AgentInfo 数据结构的位置，每个 Agent 都有一个对应的 `agent_data` account，并且如上文所述，这个 account 的地址可以被预先计算出来，是确定好的。
- `payer` 是支付「创建 PDA 所需 SOL」的账户，可以和交易发起者（也就是支付交易 gas 费用）不同，也可以相同。

在 `instruction.rs` 中，我们可以看到传进来的 `data` 是如何被解码的，在此不予赘述。

---

## State Machine

State Machine（状态机）是一个数学模型，用于描述全局的状态转换图。由于我们的 AgentInfo 中有 3 个 bool 变量 `is_registered`、`is_allowed`、`is_removed`，而每个函数中我们都对其进行了检查，所以这部分代码如果仅凭肉眼是很难理解的。因此，我们做了一个状态转换图，描述这 3 个 bool 变量在哪些函数下可以转换为别的状态，并以此来理解整个合约的逻辑。

<image>

我们和 Solidity 版本的 ATTPS 合约尽量保持了逻辑的一致性。

--

## Agent Config State

我们的 Solana 合约大部分都和 Solidity 合约保持了一致，但是关于 ConfigState 的设计，目前选择了对其进行简化。在 Solidity 中，仅需一个简单的 `mapping(address => Common.AgentConfigState) private s_agentConfigStates;` 就能够管理 Agent Config 的历史状态，但这对于 Solana 是极其困难的。首先，每个历史状态就需要 4096 bytes 的空间（一个 AgentInfo 的大小），如果我们直接使用一个 PDA 来存储一个 Agent Config State，那么至少需要 409600 来存储 100 个历史状态；但是由于 PDA 本身是不可「扩张」的，也就是说一开始我们就需要支付如此多的租金来存储数据，并且每次读取和写入都需要完整的访问这 409600 bytes 的空间（来寻找历史的 config state），这显然是不合理的。而 Solidity 中，由于 mapping 有已经设计好的动态存储机制，这些问题都被隐去了。

因此，我们抛弃了「历史状态」也就是 Config State 的概念，而是使用一个叫 `pending_settings` 的变量来存储中间状态。具体如下：

- 在 `change_agent_setting_proposal` （任何用户可以调用）时，把新的 `proposed_settings` 放入这个临时的 `pending_settings` 变量中。
- 在 `accept_agent_setting_proposal` （只有 owner 可以调用）时，把 `pending_settings` 的值赋给 `agent_settings`，并清空 `pending_settings`。
- 当 `verify` 函数被调用时，无论 `pending_settings` 是否为空，都使用 `agent_settings` 来验证。
- `pending_settings` 本身的类型是 `Option<AgentSettings>`，因此允许其中为空，也允许其中含有一个 `AgentSettings`。

这个设计参考了 Solidity 中 `Ownable2Step` 的设计哲学，即有一个 `pending_owner`，每次在 `transferOwnership` 的时候都先把 `pending_owner` 赋值为新的 owner，然后当 `acceptOwnership` 被新的 owner 调用时，再把 `pending_owner` 赋值给 `owner`。

