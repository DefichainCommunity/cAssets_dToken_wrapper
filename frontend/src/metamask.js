//import { ethers } from "./ethers.min.js";
import { ethers } from "https://cdn.jsdelivr.net/npm/ethers/dist/ethers.min.js";

let provider;
let signer;

// METAMASK
export async function connect_metamask() {
    if (!window.ethereum) throw new Error("MetaMask not installed");
    await window.ethereum.request({ method: 'eth_requestAccounts' });
    provider = new ethers.BrowserProvider(window.ethereum);
    signer = await provider.getSigner();
    return await signer.getAddress();
}

// ERC20
export async function get_token_balance(user, token) {
    const abi = ["function balanceOf(address) view returns (uint256)",
                 "function decimals() view returns (uint8)"];
    try {
        const erc20 = new ethers.Contract(token, abi, provider);
        const [bal, decimals] = await Promise.all([erc20.balanceOf(user), erc20.decimals()]);
        return ethers.formatUnits(bal, decimals);
    } catch (err) {
        // console.error("ERC20 get balance error: ",err);
        return `Error: ${err.reason || err.message}`;
    }
}

// FACTORY
export async function get_all_wrappers(factoryAddress) {
    // Factory ABI
    console.log("GetAllWrappers called");
    const factoryAbi = ["function getAllWraps() view returns (address[])"];
    const factory = new ethers.Contract(factoryAddress, factoryAbi, provider);
    // Wrapper ABI
    const wrapperAbi = ["function info() view returns (tuple(address dTokenAddress, address cAssetAddress, uint8 dTokenDecimals, uint8 cAssetDecimals, uint256 dTokenInFeeBps, uint256 dTokenOutFeeBps, address dTokenTreasury, address cAssetTreasury))"];
    const wrapAddresses = await factory.getAllWraps();
    // console.log("WrapAddresses:", wrapAddresses);
    const tokenList = [];

    for (const wrapperAddr of wrapAddresses) {
        console.log("WrapperAddress:", wrapperAddr);
        try {
            const wrapper = new ethers.Contract(wrapperAddr, wrapperAbi, provider);
            const info = await wrapper.info();

            const dTokenContract = new ethers.Contract(
                info.dTokenAddress,
                ["function symbol() view returns (string)", "function decimals() view returns (uint8)"],
                provider
            );

            const [dtoken_symbol, dtoken_decimals] = await Promise.all([
                dTokenContract.symbol(),
                dTokenContract.decimals()
            ]);

            const cAssetContract = new ethers.Contract(
                info.cAssetAddress,
                ["function symbol() view returns (string)", "function decimals() view returns (uint8)"],
                provider
            );

            const [casset_symbol, casset_decimals] = await Promise.all([
                cAssetContract.symbol(),
                cAssetContract.decimals()
            ]);


            tokenList.push({
                wrapper: wrapperAddr,
                dTokenSymbol: dtoken_symbol,
                dTokenAddress: info.dTokenAddress,
                dTokenDecimals: info.dTokenDecimals,
                cAssetSymbol: casset_symbol,
                cAssetAddress: info.cAssetAddress,
                cAssetDecimals: info.cAssetDecimals,
                fees: {
                    inBps: info.dTokenInFeeBps,
                    outBps: info.dTokenOutFeeBps
                }
            });
        } catch (err) {
            console.warn("Skipping wrapper", wrapperAddr, err);
        }
    }
    return tokenList; // Array of objects with wrapper + cAsset info
}


// ROUTER
export async function wrap_tokens(contractAddress, dToken, amount, cAsset) {
    try {
        const abi = ["function wrap(address dTokent, uint256 amount, address cAsset) external"];
        const approveAbi = ["function approve(address spender, uint256 amount) external returns (bool)",
                           "function decimals() view returns (uint8)"];
        console.log("contractAddress:", contractAddress, " dToken:",dToken, " Amount:",amount," cAsset:", cAsset);
        const erc20_contract = new ethers.Contract(dToken, approveAbi, signer);
        const decimals = await erc20_contract.decimals();
        const amount_u256 = ethers.parseUnits(amount,decimals);
        const erc20_connected = erc20_contract.connect(signer);
        const approve_tx = await erc20_connected.approve(contractAddress, amount_u256);
        await approve_tx.wait();
        const contract = new ethers.Contract(contractAddress, abi, signer);
        const connected = contract.connect(signer);
        const tx = await connected.wrap(dToken, amount_u256, cAsset);
        const receipt = await tx.wait();
        return "Wrap successful: ", receipt.hash;
    } catch (err) {
        console.error(err);
        return `Error: ${err.reason || err.message}`;
    }

}

export async function unwrap_tokens(contractAddress, cAsset, amount, dToken) {
    try {
        const abi = ["function unwrap(address cAsset, uint256 amount, address dToken) external"];
        const approveAbi = ["function approve(address spender, uint256 amount) external returns (bool)",
                           "function decimals() view returns (uint8)"];
        console.log("Unwrap on contractAddress:", contractAddress, " cAsset:", cAsset, " Amount:",amount, " dToken:",dToken);
        const erc20_contract = new ethers.Contract(cAsset, approveAbi, signer);
        const decimals = await erc20_contract.decimals();
        const amount_u256 = ethers.parseUnits(amount,decimals);
        const erc20_connected = erc20_contract.connect(signer);
        const approve_tx = await erc20_connected.approve(contractAddress, amount_u256);
        await approve_tx.wait();
        const contract = new ethers.Contract(contractAddress, abi, signer);
        const connected = contract.connect(signer);
        const tx = await connected.unwrap(cAsset, amount_u256, dToken);
        const receipt = await tx.wait();
        return "Unwrap successful: ", receipt.hash;
    } catch (err) {
        console.error(err);
        return `Error: ${err.reason || err.message}`;
    }
}
