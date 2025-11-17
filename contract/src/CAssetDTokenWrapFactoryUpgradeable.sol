// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import {CAssetDTokenWrapUpgradeable} from "./CAssetDTokenWrapUpgradeable.sol";

contract CAssetDTokenWrapFactoryUpgradeable is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    address public router;
    address public tokenTreasuryImplementation;
    address public cAssetDTokenWrapImplementation;

    address[] public allWraps;

    event CAssetDTokenWrapDeployed(
        address indexed wrap,
        address indexed dToken,
        address indexed cAsset,
        address deployer
    );

    function initialize(
        address _tokenTreasuryImpl,
        address _cAssetDTokenWrapImpl,
        address _router,
        address owner_
    ) public initializer {
        __Ownable_init(owner_);
        __UUPSUpgradeable_init();

        router = _router;
        tokenTreasuryImplementation = _tokenTreasuryImpl;
        cAssetDTokenWrapImplementation = _cAssetDTokenWrapImpl;
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    function deployWrap(
        address _dTokenAddress,
        address _cAssetAddress,
        uint8 _dTokenDecimals,
        uint8 _cAssetDecimals,
        uint256 _dTokenInFeeBps,
        uint256 _dTokenOutFeeBps
    ) external returns (address) {
        // use struct to solve Stack too deep.
        CAssetDTokenWrapUpgradeable.WrapInfo memory info = CAssetDTokenWrapUpgradeable.WrapInfo({
            dTokenAddress: _dTokenAddress,
            cAssetAddress: _cAssetAddress,
            dTokenDecimals: _dTokenDecimals,
            cAssetDecimals: _cAssetDecimals,
            dTokenInFeeBps: _dTokenInFeeBps,
            dTokenOutFeeBps: _dTokenOutFeeBps
            });
        ERC1967Proxy proxy = new ERC1967Proxy(
            cAssetDTokenWrapImplementation,
            abi.encodeWithSelector(
                CAssetDTokenWrapUpgradeable.initialize.selector,
                owner(),             // factory owner as admin
                router,              // router is the only contract to call (un)wrap
                tokenTreasuryImplementation,
                info
            )
        );

        address wrapAddr = address(proxy);
        allWraps.push(wrapAddr);

        emit CAssetDTokenWrapDeployed(wrapAddr, _dTokenAddress, _cAssetAddress, msg.sender);
        return wrapAddr;
    }

    function getAllWraps() external view returns (address[] memory) {
        return allWraps;
    }
}
