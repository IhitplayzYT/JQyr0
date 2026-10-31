pub mod Helper{
    use std::{process::exit};
    const OK:i32 = 0;
    const ERR:i32 = -1;
    const DBG_STR: &str = "Usage: jqyr0 <init|check|recreate|packer> [options]\nUse '<command> --help' for command options.";
    const PACKER_STR: &str = "packer [--config=PATH] --file=PATH | --raw-file=PATH";
    const CHECK_STR: &str = "check [--config=PATH] validates JQyr0.toml and asks the server whether an update is available.";
    const FINALISE_STR: &str = "recreate [--config=PATH] writes embedded raw files and downloads configured files.";
    const INIT_STR: &str = "init [--working-dir=PATH] --source=URL [--pull-endpoint=PATH] [--check-update-endpoint=PATH] [--file=PATH] [--raw-file=PATH]";
    const HELP_STR: &str = DBG_STR;
    
    #[derive(Debug,Clone)]
    pub enum Tool{
        Help,
        Init,
        Check,
        Finalise,
        Packer
    }

    impl From<&String> for Tool{
        fn from(value: &String) -> Self {
            match value.to_lowercase().trim(){
                "help" => {Tool::Help}
                "init" => {Tool::Init},
                "check"=> {Tool::Check},
                "compile" | "build" | "finalise" | "recreate" => {Tool::Finalise}
                "packer" | "pack" => {Tool::Packer}
                _ => {Tool::Help}
            }
        }

    }

    impl Default for Tool{ fn default() -> Self { Self::Help } }

    #[derive(Debug,Clone)]
    pub struct CLI{
        pub subtool: Tool,
        pub tool_args: Vec<String>
    }


    pub fn Help(tool : Option<&Tool>){
        println!("{}",
        if let Some(z) = tool{
            match z {
                Tool::Check => CHECK_STR,
                Tool::Finalise => FINALISE_STR,
                Tool::Help => HELP_STR,
                Tool::Packer => PACKER_STR,
                Tool::Init => INIT_STR,
            }
        }else{
            DBG_STR
        }
    );
        exit(OK);
    }


    impl CLI{
        pub fn new() -> Self{ Self {subtool:Tool::default(),tool_args:vec![]} }

        pub fn Parse_Args(&mut self){
            let args: Vec<String> = std::env::args().skip(1).collect();
            if args.len() == 0{ Help(None);}
            let k = args.first().unwrap();
            if matches!(k.to_lowercase().trim(),"--help"|"-h"){Help(None);}
            self.subtool = Tool::from(k);
            for i in args.iter().skip(1){
                    if i == "-h" || i == "--help" || i == "--HELP" || i == "-H"{
                    Help(Some(&self.subtool));
                } else{
                    self.tool_args.push(i.into());
                }
            } 
        }
    }
}
