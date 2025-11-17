// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

contract TokenTreasuryUpgradeable is Initializable, UUPSUpgradeable, AccessControlUpgradeable {
    using SafeERC20 for IERC20;

    IERC20 public token;
    uint256 public deposits;
    uint256 public balance;

    bytes32 public constant WRAPPER_ROLE  = keccak256("WRAPPER_ROLE");
    bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");

    function initialize(address wrapper, address admin, address _token) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(UPGRADER_ROLE, admin);
        _grantRole(WRAPPER_ROLE, wrapper);

        require(address(token) == address(0), "Token already initialized");
        token = IERC20(_token);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(UPGRADER_ROLE) {}

    function deposit(uint256 amount) external {
        require(amount > 0, "Treasury: zero deposit");
        token.safeTransferFrom(msg.sender, address(this), amount);
        deposits += amount;
        balance += amount;
    }

    function dispense(address to, uint256 amount) external onlyRole(WRAPPER_ROLE) {
        require(token.balanceOf(address(this)) >= amount, "Treasury: insufficient balance");
        token.safeTransfer(to, amount);
        balance -= amount;
    }

    function reclaimNotify(uint256 amount) external onlyRole(WRAPPER_ROLE) {
        balance += amount;
    }
}
