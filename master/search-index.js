var searchIndex = JSON.parse('{\
"i3nator":{"doc":"i3nator","t":[0,0,0,0,0,8,3,11,11,11,11,10,11,10,11,11,10,11,11,10,11,11,11,11,10,11,11,5,10,11,10,11,12,11,10,11,11,10,11,12,10,11,10,11,11,11,11,11,10,11,11,12,13,13,13,13,3,4,13,13,13,13,13,13,6,8,13,13,13,13,11,11,11,11,11,11,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,11,11,12,11,11,11,12,11,11,11,11,11,11,11,11,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,11,11,12,11,11,11,12,11,11,11,11,11,11,11,11,11,3,3,3,13,3,4,3,13,4,13,13,13,13,12,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,12,11,11,11,11,11,11,12,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12],"n":["configfiles","errors","layouts","projects","types","ConfigFile","ConfigFileImpl","borrow","borrow_mut","clone","clone_into","copy","copy","create","create","create","create_from_template","create_from_template","create_from_template","delete","delete","eq","fmt","from","from_path","from_path","into","list","list","list","name","name","name","ne","open","open","open","path","path","path","prefix","prefix","rename","rename","to_owned","try_from","try_into","type_id","verify","verify","vzip","0","CantBeImplemented","CommandSplittingFailed","ConfigExists","EditorNotFound","Error","ErrorKind","I3EstablishError","I3MessageError","InvalidUtF8Path","IoError","Msg","PathDoesntExist","Result","ResultExt","TextOrKeyInputFailed","TomlError","UnknownConfig","Utf8Error","backtrace","backtrace","borrow","borrow","borrow_mut","borrow_mut","chain_err","chain_err","chain_err","description","description","description","extract_backtrace","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from","from","from","from","from_kind","from_kind","into","into","iter","iter","kind","kind","new","source","to_string","to_string","try_from","try_from","try_into","try_into","type_id","type_id","vzip","vzip","with_boxed_chain","with_chain","with_chain","Layout","borrow","borrow_mut","clone","clone_into","copy","create","create_from_template","delete","deref","eq","fmt","from","from_path","into","list","list","name","name","ne","open","path","path","prefix","rename","to_owned","try_from","try_into","type_id","verify","vzip","Project","borrow","borrow_mut","clone","clone_into","config","copy","create","create_from_template","delete","deref","eq","fmt","from","from_path","into","list","list","name","name","ne","open","path","path","prefix","rename","start","to_owned","try_from","try_into","type_id","verify","vzip","Application","ApplicationCommand","Config","Contents","Exec","ExecType","General","Keys","Layout","Managed","Path","Text","TextNoReturn","applications","args","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","clone","clone","clone","clone","clone","clone","clone","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","command","commands","default","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","eq","eq","eq","eq","eq","eq","eq","exec","exec_type","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","general","into","into","into","into","into","into","into","layout","ne","ne","ne","ne","ne","ne","program","timeout","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","type_id","vzip","vzip","vzip","vzip","vzip","vzip","vzip","working_directory","working_directory","workspace"],"q":["i3nator","","","","","i3nator::configfiles","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","i3nator::errors","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","i3nator::layouts","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","i3nator::projects","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","i3nator::types","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["Module consolidating common functionality between …","Errors, using <code>error-chain</code>.","Module for layout handling.","Module for project handling.","The types in this module make up the structure of the …","Helping type to consolidate common functionality between …","Helping type to consolidate common functionality between …","","","","","Create a copy of the current configfile, that is a copy …","","Create a configfile given a <code>name</code>.","","Create a configfile given a <code>name</code> and <code>prefix</code>.","Create a configfile given a <code>name</code>, pre-filling it with a …","","Create a configfile given a <code>name</code>, pre-filling it with a …","Delete this configfile from disk.","","","","","Opens an existing configfile for a given path.","","","Get a list of all configfile names for a given prefix.","Get a list of all configfile names.","","Returns the name of this configfile.","","The name of the configfile.","","Opens an existing configfile using a <code>name</code>.","","Opens an existing configfile using a <code>name</code>.","Returns the path to the configfile.","","The path to the configfile.","Return the prefix associated with this type of configfile.","","Rename the current configfile.","","","","","","This verifies the project’s configuration, without …","","","The kind of the error.","An error that occurs if a trait-function is called that …","An error that can occur when splitting a string into a …","An error that occurs if a project under the same name …","An error that occurs when the default editor is not …","The Error type.","The kind of an error.","Error caused by <code>i3ipc</code>, on establishing a connection.","Error caused by <code>i3ipc</code>, on sending a message.","An error that occurs when a <code>Path</code> (i.e. <code>OsStr</code>) cannot be …","Error mapping to <code>std::io::Error</code>.","A convenient variant for String.","An error that occurs if a specified path does not exist.","Convenient wrapper around <code>std::Result</code>.","Additional methods for <code>Result</code>, for easy interaction with …","An error that occurs if text or key-presses could not be …","Error caused by <code>toml</code>, on deserializing using Serde.","An error that occurs if a project does not exist under a …","Error mapping to <code>std::str::Utf8Error</code>.","Returns the backtrace associated with this error.","","","","","","If the <code>Result</code> is an <code>Err</code> then <code>chain_err</code> evaluates the …","Extends the error chain with a new entry.","","","A short description of the error. This method is …","A string describing the error kind.","","","","","","","","","","","","","","","","","","","","Constructs an error from a kind, and generates a …","","","","Iterates over the error chain.","","Returns the kind of the error.","","","","","","","","","","","","","Construct a chained error from another boxed error and a …","Constructs a chained error from another error and a kind, …","","A structure representing a managed i3-layout.","","","","","","","","","","","","","","","Get a list of all layout names.","","","The name of the layout.","","","","The path to the layout configuration.","","","","","","","","","A structure representing a <code>i3nator</code> project.","","","","","Gets the project’s configuration, loading and storing …","","","","","","","","","","","Get a list of all project names.","","","The name of the project.","","","","The path to the project configuration.","","","Start the project.","","","","","","","The applications configuration.","The command used for starting an application.","This is the parent type defining the complete project …","The layout is provided directly as a string.","Commands to execute or keys to simulate after application …","Defines how the commands in <code>Exec</code> should be interpreted.","The general configuration section.","Interpret the commands given as key presses.","This holds the layout, in multiple formats.","The name of a managed layout","The layout is provided as a path.","Interpret the commands given as separate text-lines, …","Interpret the commands given as text, but do not input a …","The applications configuration list.","A list of arguments to pass to the executable.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","The command used for starting an application.","List of text or keys to input into the application.","","","","","","","","","","","","","","","","Commands to execute or keys to simulate after application …","Defines how the commands above should be interpreted.","","","","","","","","","","","","","","","The general configuration section.","","","","","","","","The layout to append to a workspace.","","","","","","","The executable to start.","Specify a timeout after which a command has to be …","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","The working directory defines in which directory-context …","The working directory defines in which directory-context …","If the workspace is <code>Some</code>, <code>i3</code> will be instructed to open …"],"i":[0,0,0,0,0,0,0,1,1,1,1,2,1,2,1,1,2,1,1,2,1,1,1,1,2,1,1,0,2,1,2,1,1,1,2,1,1,2,1,1,2,1,2,1,1,1,1,1,2,1,1,3,4,4,4,4,0,0,4,4,4,4,4,4,0,0,4,4,4,4,3,3,3,4,3,4,5,3,3,3,3,4,3,3,3,4,4,3,3,3,3,3,3,3,3,3,4,4,4,4,3,3,3,4,3,3,3,3,3,3,3,4,3,4,3,4,3,4,3,4,3,3,3,0,6,6,6,6,6,6,6,6,6,6,6,6,6,6,0,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,0,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,0,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,0,0,0,8,0,0,0,9,0,8,8,9,9,10,11,10,12,8,13,11,14,9,10,12,8,13,11,14,9,10,12,8,13,11,14,9,10,12,8,13,11,14,9,13,14,11,10,12,8,13,11,14,9,10,12,8,13,11,14,9,13,14,10,12,8,13,11,14,9,10,12,8,13,11,14,9,10,10,12,8,13,11,14,9,12,10,12,8,13,11,14,11,14,10,12,8,13,11,14,9,10,12,8,13,11,14,9,10,12,8,13,11,14,9,10,12,8,13,11,14,9,10,12,8,13,11,14,9,12,13,12],"f":[null,null,null,null,null,null,null,[[]],[[]],[[],["configfileimpl",3]],[[]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[["configfileimpl",3]],["bool",15]],[[["formatter",3]],["result",6]],[[]],[[],["result",6]],[[],["result",6]],[[]],[[],[["vec",3],["osstring",3]]],[[],[["vec",3],["osstring",3]]],[[],[["vec",3],["osstring",3]]],[[],["string",3]],[[],["string",3]],null,[[["configfileimpl",3]],["bool",15]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["pathbuf",3]],[[],["pathbuf",3]],null,[[],["osstr",3]],[[],["osstr",3]],[[],["result",6]],[[],["result",6]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["result",6]],[[],["result",6]],[[]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[],[["option",4],["backtrace",3]]],[[],[["option",4],["backtrace",3]]],[[]],[[]],[[]],[[]],[[],[["error",3],["result",4]]],[[],["error",3]],[[]],[[],["str",15]],[[],["str",15]],[[],["str",15]],[[["error",8]],[["option",4],["internalbacktrace",3]]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["error",3]]],[[]],[[["string",3]]],[[["str",15]]],[[["errorkind",4]]],[[["utf8error",3]]],[[["error",3]]],[[["establisherror",4]]],[[["messageerror",4]]],[[["error",3]]],[[["str",15]]],[[["string",3]]],[[]],[[]],[[["errorkind",4]],["error",3]],[[]],[[]],[[],["iter",3]],[[],["iter",3]],[[]],[[],["errorkind",4]],[[["errorkind",4],["state",3]],["error",3]],[[],[["option",4],["error",8]]],[[],["string",3]],[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[["error",8],["box",3]],["error",3]],[[],["error",3]],[[]],null,[[]],[[]],[[],["layout",3]],[[]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["configfileimpl",3]],[[["layout",3]],["bool",15]],[[["formatter",3]],["result",6]],[[]],[[],["result",6]],[[]],[[],[["vec",3],["osstring",3]]],[[],[["vec",3],["osstring",3]]],[[],["string",3]],null,[[["layout",3]],["bool",15]],[[],["result",6]],[[],["pathbuf",3]],null,[[],["osstr",3]],[[],["result",6]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["result",6]],[[]],null,[[]],[[]],[[],["project",3]],[[]],[[],[["config",3],["result",6]]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["result",6]],[[],["configfileimpl",3]],[[["project",3]],["bool",15]],[[["formatter",3]],["result",6]],[[]],[[],["result",6]],[[]],[[],[["vec",3],["osstring",3]]],[[],[["vec",3],["osstring",3]]],[[],["string",3]],null,[[["project",3]],["bool",15]],[[],["result",6]],[[],["pathbuf",3]],null,[[],["osstr",3]],[[],["result",6]],[[["osstr",3],["str",15],["option",4],["i3connection",3],["option",4]],["result",6]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["result",6]],[[]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["config",3]],[[],["general",3]],[[],["layout",4]],[[],["application",3]],[[],["applicationcommand",3]],[[],["exec",3]],[[],["exectype",4]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,null,[[],["applicationcommand",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["config",3]],["bool",15]],[[["general",3]],["bool",15]],[[["layout",4]],["bool",15]],[[["application",3]],["bool",15]],[[["applicationcommand",3]],["bool",15]],[[["exec",3]],["bool",15]],[[["exectype",4]],["bool",15]],null,null,[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,[[["config",3]],["bool",15]],[[["general",3]],["bool",15]],[[["layout",4]],["bool",15]],[[["application",3]],["bool",15]],[[["applicationcommand",3]],["bool",15]],[[["exec",3]],["bool",15]],null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,null,null],"p":[[3,"ConfigFileImpl"],[8,"ConfigFile"],[3,"Error"],[4,"ErrorKind"],[8,"ResultExt"],[3,"Layout"],[3,"Project"],[4,"Layout"],[4,"ExecType"],[3,"Config"],[3,"ApplicationCommand"],[3,"General"],[3,"Application"],[3,"Exec"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};