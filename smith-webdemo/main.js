import init, {generate_ts, show_imps_map,generate_rust} from "./pkg/smith_webdemo.js"
import hljs from 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/es/highlight.min.js';
//  and it's easy to individually load additional languages
import rust from 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/es/languages/rust.min.js';
import typescript from 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/es/languages/typescript.min.js';
hljs.registerLanguage('rust', rust);
hljs.registerLanguage('typescript',typescript);

let schema = `struct Pair<A,B>{
    first:  A
    second: B
}

enum Optional<T>{
    Some(T)
    None
}

enum WebEvent {
    PageLoad
    KeyPress(u8)
    OnInput(Optional<WebEvent>)
    Click(Pair<u32,u32>)
}`

let schemafield = (()=>{
    let root = document.getElementById("schema");
    let c = ()=>{
    };
    root.addEventListener("input", c)
    root.value = schema
    root.classList.add("codefield")
    return {el: root,setValue:(v)=>{
        root.innerText = v;
        c()
    }}
})();


let impsmap = createField("js")
let rustcode = createField("rs");
let tscode = createField("ts");

function createField(type){
    let root = document.createElement("pre");
    let code = document.createElement("code");
    code.classList.add("language-" + type);
    root.classList.add("output")
    root.append(code);

    return {el: root,setValue:(v)=>{
        code.innerHTML = v;
    }}
}

function recompile(){
    schema = schemafield.el.innerText
    try{
        impsmap.setValue(show_imps_map(schema));
        tscode.setValue(generate_ts(schema));
        rustcode.setValue(generate_rust(schema));
        /** @type {Error} */
    }catch(e){
        alert("An error occured while compiling the schema")
        throw e;
    }

    hljs.highlightAll({ ignoreUnescapedHTML: true });
}

window.recompile = recompile;

async function main(){
    await init();

    document.getElementById("outlist").append(impsmap.el,rustcode.el,tscode.el)
    schemafield.setValue(schemafield.el.value)
    recompile();
}
main()