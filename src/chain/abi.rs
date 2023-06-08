use ethers::prelude::abigen;

abigen!(
    Bridge,
    r#"[
        event Stake(address indexed account, address token_addr, uint256 amount)
        function mint(address account, address token_addr, uint256 amount) external override onlyOwner returns (bool success)
    ]"#,
);
