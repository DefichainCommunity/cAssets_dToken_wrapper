//import { ethers } from "./ethers.min.js";
import { ethers } from "https://cdn.jsdelivr.net/npm/ethers/dist/ethers.min.js";

let provider;
let signer;

// METAMASK
export async function js_connect_metamask() {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        provider = new ethers.BrowserProvider(window.ethereum);
        signer = await provider.getSigner();
        const addr = await signer.getAddress();
        return {
            ok: true,
            value: JSON.stringify(addr)
        };
    }catch (err) {
        console.error(err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }
}

// ERC20
export async function js_get_token_balance(user, token) {
    const abi = ["function balanceOf(address) view returns (uint256)",
                 "function decimals() view returns (uint8)"];
    try {
        const erc20 = new ethers.Contract(token, abi, provider);
        const [bal, decimals] = await Promise.all([erc20.balanceOf(user), erc20.decimals()]);
        return {
            ok: true,
            value: JSON.stringify(ethers.formatUnits(bal, decimals))
        };
    } catch (err) {
        console.error(err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }
}

// FACTORY
export async function js_get_all_wrappers(factoryAddress) {
    // Factory ABI
    console.log("GetAllWrappers called");
    try {
        const factoryAbi = ["function getAllWraps() view returns (address[])"];
        const factory = new ethers.Contract(factoryAddress, factoryAbi, provider);
        // Wrapper ABI
        const wrapperAbi = ["function info() view returns (tuple(address dTokenAddress, address cAssetAddress, uint8 dTokenDecimals, uint8 cAssetDecimals, uint256 dTokenInFeeBps, uint256 dTokenOutFeeBps))"];
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
        return {
            ok: true,
            value: JSON.stringify(tokenList , (key, value) =>
                typeof value === "bigint" ? value.toString() : value
            )
        };
    } catch (err) {
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };

    }
}


// ROUTER
export async function js_wrap_tokens(contractAddress, dToken, amount, cAsset) {
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
        return {
            ok: true,
            value: JSON.stringify(`${receipt.hash}`)
        };
    } catch (err) {
        console.error(err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }

}

export async function js_unwrap_tokens(contractAddress, cAsset, amount, dToken) {
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
        return {
            ok: true,
            value: JSON.stringify(`${receipt.hash}`)
        };
    } catch (err) {
        console.error(err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }
}
