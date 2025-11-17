// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Script.sol";
import {TokenTreasuryUpgradeable} from "../src/TokenTreasuryUpgradeable.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";


contract TestnetDeposit is Script {
    function run() external {
        vm.startBroadcast();
        address treasury = address(0x9D4F577fC58c885DB6B5f5A1a8fFdCECcAB5C40a);
        address dToken = address(0xff0000000000000000000000000000000000005B);

        IERC20(dToken).approve(treasury, 1_000_000 ether);
        TokenTreasuryUpgradeable(treasury).deposit(1_000_000 ether);
        vm.stopBroadcast();
    }
}
