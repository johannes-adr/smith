import init, { SmithJS } from "../pkg/smith_js.js"

async function main() {
    await init(); //Wait for WASM init
    let schemaData = await fetch("./schema.smith");
    let smith = new SmithJS(await schemaData.text());
    let json = {
        leiter: {
            tag: "Some", val: {
                name: {
                    vorname: "Max",
                    nachname: "Müller"
                },
                alter: 42
            },

        },
        matrikelnr: [12345, 52221]
    }

    let bytes = smith.serialize(json, "Kurs")
    let back = smith.deserialize(bytes, "Kurs");


    document.body.innerHTML = `
        Original: <br><pre>${JSON.stringify(json, null, 2)}</pre><br>
        Bytes: <code>[${bytes}]</code><br><br>
        Deserialized: <br><pre>${JSON.stringify(back, null, 2)}</pre><br>
    `
}

main()


