#[derive(Clone)]
pub struct Command {
    pub name: &'static str,
    pub cmd: for<'a> fn(&'a mut crate::Repl, &[String]) -> color_eyre::Result<()>,
    pub args: &'static [(&'static str, &'static str)],
    pub help: &'static str,
}

const SHOW_CMD: Command = Command {
    name: "show",
    cmd: |_r, input| {
        match input[0].as_str() {
            // "scope" => r.cu.runner.as_mut().unwrap().scope.order[0]
            // .iter()
            // .for_each(|(name, obj)| {
            //     println!("let {name} = {}", obj.with(|o| o.to_pretty_class_str(0)))
            // }),
            "env" => std::env::vars().for_each(|(k, v)| println!("{k:?} = {v:?}")),
            _ => eprintln!("what is {input:?}?"),
        }
        Ok(())
    },
    args: &[("thing", "thing to be shown")],
    help: r#"shows something like "scope", etc. "#,
};

const HELP_CMD: Command = Command {
    name: "help",
    cmd: |r, input| {
        r.commands.get(&input[0]).map_or_else(
            || eprintln!("unknown command {input:?}"),
            |cmd| {
                println!("{}: {}", cmd.name, cmd.help);
                println!("args:");
                cmd.args
                    .iter()
                    .for_each(|(arg, desc)| println!("\t{arg}: {desc}"));
            },
        );
        Ok(())
    },
    args: &[("cmd", "show the help message for that command")],
    help: "show this message",
};

const ENV_CMD: Command = Command {
    name: "env",
    cmd: |_r, input| {
        std::env::set_var(input[0].clone(), input[1].clone());
        Ok(())
    },
    args: &[
        ("var", "variable to be setted"),
        ("value", "value of the variable"),
    ],
    help: "sets an environment variable",
};

const SET_CMD: Command = Command {
    name: "set",
    cmd: |r, input| {
        fn set_arg<T: std::str::FromStr>(setting: &mut T, v: &str) -> color_eyre::Result<()>
        where
            <T as std::str::FromStr>::Err: 'static + std::error::Error + Sync + Send,
        {
            *setting = T::from_str(v)?;
            Ok(())
        }

        match input[0].as_str() {
            "prompt" => set_arg(&mut r.settings.prompt, &input[1])?,
            "show_ast" => set_arg(&mut r.settings.show_ast, &input[1])?,
            "show_tokens" => set_arg(&mut r.settings.show_tokens, &input[1])?,
            "show_output" => set_arg(&mut r.settings.show_output, &input[1])?,
            "bench" => set_arg(&mut r.settings.bench, &input[1])?,
            "run" => set_arg(&mut r.settings.run, &input[1])?,
            _ => Err(crate::Err::UnknownSetting(input[0].clone()))?,
        };
        Ok(())
    },
    args: &[
        ("setting", "setting to be setted"),
        ("value", "new value of that variable"),
    ],
    help: "sets something of the repl",
};

const CMDS_CMD: Command = Command {
    name: "cmds",
    cmd: |r, _| {
        for Command {
            name: n, help: h, ..
        } in r.commands.values()
        {
            println!("{n}: {h}");
        }
        Ok(())
    },
    args: &[],
    help: "show the available commands",
};

const QUIT_CMD: Command = Command {
    name: "quit",
    cmd: |r, _| {
        r.should_exit = true;
        Ok(())
    },
    args: &[],
    help: "quits the repl",
};

pub const COMMANDS: &[Command] = &[SHOW_CMD, HELP_CMD, ENV_CMD, SET_CMD, CMDS_CMD, QUIT_CMD];
