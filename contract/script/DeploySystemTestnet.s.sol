// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Script.sol";
import "../src/TokenTreasuryUpgradeable.sol";
import "../src/CAssetDTokenWrapUpgradeable.sol";
import "../src/CAssetDTokenWrapFactoryUpgradeable.sol";
import "../src/CAssetDTokenWrapRouterUpgradeable.sol";
import "../mocks/MockERC20.sol";

contract DeploySystem is Script {
    function run() external {
        vm.startBroadcast();

        address cAsset = address(0x37386064e05d89FA6F4c9c1d2C05AbD6388aD750);
        address dToken = address(0xff0000000000000000000000000000000000005B);
        // Step 2: Deploy logic implementations
        TokenTreasuryUpgradeable tokenTreasuryImpl = new TokenTreasuryUpgradeable();
        CAssetDTokenWrapUpgradeable wrapImpl = new CAssetDTokenWrapUpgradeable();
        console.log("TokenTreasury impl:", address(tokenTreasuryImpl));
        console.log("CAssetDTokenWrap impl:", address(wrapImpl));

        // Step 3: Deploy router
        CAssetDTokenWrapRouterUpgradeable router = new CAssetDTokenWrapRouterUpgradeable();
        router.initialize(msg.sender);
        console.log("Router:", address(router));

        // Step 4: Deploy factory (UUPS)
        CAssetDTokenWrapFactoryUpgradeable factory = new CAssetDTokenWrapFactoryUpgradeable();
        factory.initialize(address(tokenTreasuryImpl), address(wrapImpl), address(router), msg.sender);
        console.log("Factory:", address(factory));

        // Step 5: Deploy a new wrap via factory
        address wrapAddr = factory.deployWrap(dToken, cAsset, 18, 18, 30, 20);
        console.log("Wrap instance:", wrapAddr);

        // Step 6: Register with router
        router.registerWrapper(wrapAddr, dToken, cAsset);

        address dTokenTreasury = CAssetDTokenWrapUpgradeable(wrapAddr).dTokenTreasury();
        console.log("LUSDC instance:", dTokenTreasury);

        console.log("Setup complete!");

        vm.stopBroadcast();
    }
}
