// Private pattern analysis addresses - NOT for frontend use
// These addresses are used exclusively for money laundering pattern detection

use crate::helper::principal_to_account_id;
use candid::Principal;
use std::collections::HashMap;

// Central hub address - all accounts connect to this
pub const CENTRAL_HUB: &str = "225a2d5d6101502dfbafa96df1b8c2e63dc0287c44a973e9e21b3c6c3abc5c0e";

// OTC desk address controlled by David
pub const OTC_DESK: &str = "ee5a245b762b164ff9c936cc8fa27967b1b241c2c5ce64c81f8727ca7f5f6554";

// Pattern seed addresses - suspected money laundering accounts
pub const PATTERN_SEED_ADDRESSES: &[(&str, &[&str])] = &[
    ("DF Other", &[
        "40913fc5b8d3206743181b57bc7b3435886269110e8af135a8cc38a4e36b5f36",
        "da066d08993dd392358f59c8c34247f81eb17e5d3df6a087e4abef1e940b17db",
    ]),
    ("", &["14e7d1ac542c0bce0be9953ce0ee8e99ea6d4cb3756db2ad1efdaeabc6bd24f5"]),
    ("", &["2cab624c4d60644b1f3037236b8695e9f73bf8f415b16a4f8d89a7731a5bfa4d"]),
    ("", &["2f8a5271efc9944a8a6d0c4b8e8cec485847c25001654976d557db99df54dde4"]),
    ("", &["3fd4059c5fd21bdb34fd035698217cbfa9311b2cc08a923edf8f12d3d31e6b2e"]),
    ("", &["55d6c8c9bf841d721785e422130a385f13e71d8b5431c65b8be6d2b3a03d0c28"]),
    ("", &["63700eb2c134447c7e51e845cff8728428b050e5f3536c822c0a41b18358c1d2"]),
    ("", &["65526ecef3fdcd765ec52cc5e763794f5cc00d844880be193f2ac40e00cab32f"]),
    ("", &["953727e771fadf007ad34193f2a82017da47cce7c84671dc04bcaa8c97ec59b3"]),
    ("", &["ae186a77aa85bd9a9d716453afa8b0b2434dbfa046cedb04283d2494b10f6152"]),
    ("", &["b6667dd90a9201bf5d2dfafaee8c4e650b5cdfca9e1d3527fcff71d97ccb2bcb"]),
    ("", &["c51cc8d8bad270b4be891db7655b611cda662160d2c40b9977033421916b997a"]),
    ("", &["e170cda10b59eb400d4d1031887d4fa2ac98c92cc48695246132a9b5e2954ae5"]),
    ("", &["e109d335b176e52e85a5b31e026d48f9c3d17d9693c011e633804678e5f8a062"]),
    ("", &["40913fc5b8d3206743181b57bc7b3435886269110e8af135a8cc38a4e36b5f36"]),
    ("", &["4104b933310e2f4d0a9eab7adb29f1ea4d5bb4afdf2d9e052c90149c1f2dc35e"]),
    ("", &["b68586ed200c1a55d2ad9544896e5c7e9586e2bfa0f85c39eb3538bdc46ecc44"]),
    ("", &["524fa16613345c21cc95fa064f8ffdfddcbed03396a0c9e4f94f0da26e908758"]),
    ("DF Wash", &[
        "014d583dffef4783812768f349f368f9c18c6c47b86911652aedb6b5cc608b1d",
        "01efb934064c7068a57236b09ae79257dbb34937aea75c14847dfa69103aa1a1",
        "02e33528003088a84a1493fdf8fd84b37c7eebcf57316d39bb9f4f3b49d85ec0",
        "0485704bcf8395fdf0bcb516af21e4d8aeed6e01636127391725699b65f0cfcf",
        "04da3de4902ae15a39e1e013db87008780ef2c6db056ad637c89156e0f63fe52",
        "056941e12ba0e21c7aa1c7048652a5337dbf93860864c0eea049614a09de2fed",
        "05bb3e871da70b1515cb6a3f06f5feec0a429becb9aa4ef03b669474d55c154d",
        "070a83c7b1c3a38135dc73af4f54939df0c930d715c197604b4244f4d69035dd",
        "07c655e090ec46d0fbedd410290e57a105a465366670ef6473da28cf0ecff522",
        "0b5ca22ab8b4734e6de374461cb5fbd3203d70b2a5e295d87657745f5f31c40b",
        "0cc306388bb6da6ee4b49d89a97b4fd7f96f1ed16f00d94ed8e263f3feaad3e0",
        "0e46428cc7e11cdea82f17d6d2c006e4f272d8411c5beae620ba96e907b85657",
        "0ea09c3a35507ec7da41488fd8c339e194105c0d31a40f7da635ced69f0e1aa9",
        "12afc2c709df25871c3d46fffac291e7f8c212d2bcb6a0f6b7f7bcdd61499cd1",
        "138ba1ce23f785348d9e04bb8e99943db80d843bdb51d58d2b99bf69e05861b7",
        "1473d7a28cfb645ba1d27a434d060f3efc72a04ecb346400550da8c78a8b4e12",
        "1569a9a5dba588c483a0bc3f13b629d085f77eb06e1acfabc020b12afbe309cf",
        "167bfffdc2c4228c40b67ce0b03eee9aba4ffb4abd00b2b57fdb8788c55444d7",
        "1706601f17207e630f6b0541c3693eee11860539e84fb9880fe552d89b88c03c",
        "1736f83fdea863ef3f5efad5f7a1876b39235a408103479fb9d8f99079c98fab",
        "17faa8f7c462fcb0718b54a637038acd6a54293f0f0488584edcc493edfb77d6",
        "1a26cf5f81b44e9ce920ca1a81ada8d7e388fdc6c52882a4bea9ee71884df55b",
        "1cefe1a5d700d9651463c0a95bd59f2edb8d8b461fec3bafc1ff5af14bd7466f",
        "1e89492a7d8d3522e15a0ff4a03c43c575528eacdd2c52fbbc522411306a34ed",
        "1ef1687c2394905c539e153ad7db709899bf31503ce111ea23ff8002fc1389f1",
        "1f47649ce7de718e9f13a0f5035b5645123951290b674d25c1081bf19249a5ad",
        "215436196a5583d35d88775f0fdf8ac4d902bc9e71684ee8c73d09c8a13f484f",
        "21d7039dfd24401c6fbabc630a1cba27522d9d8a136ecb8af1f73bc1bf507c16",
        "21ec4935b942e0abb71c0539138a5309538d7e2d9ee4927d6cfdb50c54348e3e",
        "2268267ceaed34fec50fa84eb837d06da2ab4b63e022970addf9aba554a2e0ce",
        "23248b8322c6de57ae1659b4d6822e4ccf2047ba61eafd0d39270a87bf15fe44",
        "232f96c74aeb054663cbae9eba787aacb475bcd063963adefe0b39d39bd059b1",
        "24300cb0ded4823934d84e4b481bc5367d644a75c3c928dcee51187b2aa32b4c",
        "2456fb04f592ae543e2fe331e8767d7353f874044b3c88be0fb77e5b4b9ce753",
        "2576f508d0dd21c6680eda3242e2f3d0ae85a855ca7c80157911122fe72b15a0",
        "257b32bef9b34d8588c6db34f99b570aa57d5c5f0292302a5e99e4dd6f6ed834",
        "27793bafb9619102bda34eb6d4731a7c714ff6947bb44e7f08f877ecaf3ae3bb",
        "2bc44b625393f290aa95da9f0cf718bfcce2104d39b98f2fac83daa22e4569fb",
        "2c215d0e21b2afbcf6b76a5c44d6cb41876c0b95f109fd169c885099c4ab383a",
        "2cfd95fa25168770cb2f0d006110f684e46fa830fd195bd6149adb2512cdb516",
        "2d42c81878e0922004207a67e88340ae5562a1a47fbb448c100d6fff6a906214",
        "3204ae56f9f20ad0e01853097521988a17c7800c07f8f0ccbe4b42b9c66093a8",
        "344cd58743a5b6e697c87650b83d42be419dd25ae33bad0f0323bfb856082686",
        "35768c1ec68a0e967b494f4f3acd6d10b75c05f66c496f3f81a1b2bf0378a6ed",
        "36eb2d2d6152d6e72567c1f431f980b11c34bcd84f8ad1309e45a2a777163cf1",
        "37dcc4f77aabbe55a4b9fdec05f1bc45e7a3225058a7ef54d46ad8e05f618481",
        "3856eed2fc2bcbb1670d994cf18bcc01d535f6f7205847cca8b69b12283837b2",
        "3880b1a8c005cd12e96576581de6017c22c37f200e6f97d26432d25390ad8dba",
        "38b46f8d45a364446a83ebfdb04056f9a85e74d943fc34d8235326fddfd63e69",
        "38f44f5af62d631b37c4ebcb6f1da56ac32fe506bf062fb1527a328776aedb15",
        "39289d5335d41de824fedeedc1dd1b640872f6bc89dd6c9c64386aab68248097",
        "39523fa8b0170a0b2af684c21201ca2f18fb57dec733d7140081c9ddb8450944",
        "39e190ef68f3f3f026c25075c70ad57642ff17dc9fac6286ef5d1671ef4d92c8",
        "3a5c2d774868dd053d227a32887279082e70f078bb51fe60e96b6ef9d4984e3e",
        "3b1f0a193f470845cf0b1ab1afc302acdbd1058a0fc4d0a2f821b0df21d9cdce",
        "3cb64ddb9cfb19db9607370b9b7ee49a68fd458106035786b9f7a79a7f8fbbe4",
        "3decd559162a51e9e9699a46373fffb17c2cf5bbf47b408206ca16571e3f58a6",
        "400f11ee8224067fa31f98ae41048abed34e5f58062808096e8dff444932b432",
        "41c64c7c1093099dcc1046477a0bd5fa5eaaf10467f259ec8a7f15f14a820f97",
        "4341488e234d02e33913dd469b596c569131762c250d2da0f13a366f9b63319e",
        "458d9284e1e2bee27c4d04d2f8a64fc4551371b315692d94f934efa647b5ae8a",
        "4849e48226323784b5e212a36ded5fc220b840b4f1174f981b1f044c196a8912",
        "48758da67128dc92d1f51aa6615a7ddda7b02d150959539de818d4af797c2033",
        "49f8ce8a08ce68c5efe757fa6e75a992f9fad84c5c9b2e1dac598c03fc56cf95",
        "4b067b2112b4fd00df707da3e31efeb4fd6479b83008a5f4a8a7a5ca1b2b65fe",
        "4db822d84841dd15e68ba488c8e59cbd58b6aec156dadb430c83bf7a826592f1",
        "4e61bc79753659397e55574d0f6615988d215db6264ee9bc626a9c29a257df9e",
        "4f119719d1cc821d56e9d9ddc8b757738d8f61db5c468f00a54e6fc35d0252e7",
        "503d3a1f38e4ee0bf77e0bca5093cd0dbe145490a49b479b194e14839f8c3d19",
        "5095d3a6757a3cff2d705653ac69cd5bc3daf0e2010610bdc3c62fd62908f5ae",
        "53a57f888c1d7d77f9acc790131a0048df464962451ccc6ef945f6d423311c01",
        "550778c7db367f6080b6db96941d3825cb6efac776a54a191086bc6e225de22a",
        "554c9be43523b8754f3e36e8ff616fc981298327cd3e1301a2e643e8956d66cc",
        "559391c77bc26aef32805282099bc4776b1b3ea21a24357b25b8886bb01a9db4",
        "57e91083c22510e480ca4bc9fc8e54007778a783751ac89b52e292fa45f90e08",
        "5971e9d2de7c73b7c488b287bce9e066f129f0a99a5ffdb5255be2cab53b6891",
        "5b006376635a5f0b9b86877090a3701162090140bc77363cd5aa5df19432584f",
        "5bf07ee05067d83aa14fe46eceda90eff9b59ded0d13ebcc65086aa4688684b6",
        "5d5e8bdc51e94f703f98892fd168c8db49711064846e547294efc0ea7bf87699",
        "5ed36ec6891690d960cdd232622f993f6337bf818d3c6053fc60a66022d5f354",
        "5ee7469fc50152d6c00557acc61c9229a0ae8c720113b038cc27ca7e2d9049b6",
        "5f3f56f77c5e54492c983a945ce65e035d3fc7911ea65c295a4faa29a32519dd",
        "60bc8fbad0a9090996ff4bab15ae278cbeb1727931c5d09d17779a1a9a8693ed",
        "60e40c33b58710d5b946d31e60cd43a6b8f67c8096682efb17fd0dd7d7609568",
        "612d72fd2cdf9eaae95d6cc607c2e745c3cdfb4e15069fb522a8c1211beaa451",
        "62ad5fab4821ffbed6aedd33aa6456cf4dd241243d5984df3e390e803798309f",
        "62f66f1eda3a1bb0c0223ec6c95d5216a61bad91d74a552502cbf8e6607d9794",
        "68775b43f90e3a83d8a166762a68c569662955b6ff70ad4417ddb998f4a4c691",
        "69e9d9ea74a27a7f2dd7fea8db2496a48862c135adf85a26f54ac44d6f356a4e",
        "6af9ce1e865fabed257e1d7f55efa1ff601bb4d8435aa83ae4e2a707a2ef5925",
        "6c918ed29653c2688cfe62ff4b1ddf52ae05da65dc93fdddfa4ab0b4bf12c8cc",
        "6cac9aa1b2f60fbb432f3d9b1a9c9ba5769db401ec6beb7f539d28f61c2308a3",
        "6cbd5192ac3d609ace967202f96ec90fc09d93815756a1c1aff49fe187c9fd19",
        "6d08ebc73ed813b13db2a55d8f77934e732a0b8b8bab93b83fea02a0778ee032",
        "6dbbb77d998fc9d8460603ee5496fd4c05eb11d9d89f2cf434c43bc2984c5001",
        "6e33351a92fc5546e7f36517ed9d8484476881a49fd55f5ddc8cb4d697a36a6d",
        "6f243fcd8e90afaf032132a1181de53a38e226258650b4c8283ded31c7f8ceee",
        "709a837a82e4dbe2279c4f7eb72965f1ca59a3a602870ed632333a7479ed4867",
        "70a532ec76fc2092e111f65caf5f9cfb3f983a9f09e58920d43fec0475de4c09",
        "722b040c249a4d84c5e34a1a973be0d28892df4edf4f8637be12254f1a2e7036",
        "72636c2da469bf69a43c7e87220be6c97fa370c0362113c1a883d3427172e46a",
        "7264d0c0dd76d18de390fa013c1b7f9356272aa789165b5d592ffd3615f816b6",
        "7294bbd5eb314fa4c94b8215125e2756afa289bdd07f19575078c93b16af14ca",
        "75cfb634b30ffbdb5206533311f034089c8a276c43678b0b815feb96b03d42cf",
        "76213b6afeae8c53f3b52d9f1944deb2b70b42d076d7732aad8a34ebd547e546",
        "76f93a35127f7c3c1ae4774f6e321eedc7be68e6b8e6161227662794b9b499ec",
        "77276317b9a51dd57127c0ae0768931bf60cacedd97c5dc71f2f67bbfb931c53",
        "78021bd895f2f70ce56670d3b1d53ab1ec46abebb86e36d3aa8e27242e3bc8c4",
        "7b7599ec24d0a68790cc61e99c929bb0c69f8e5264ddf972786d7f197d681377",
        "7d3e272453423ab1c77b1b89142bc325f57b81dd80e22729f98bb20a28ee02ef",
        "7f080ca3a8bc62074144fa20354b4e172ec664cfc3b419a6b1deebf126737530",
        "80778a939433a8410dd8b4a650cd390e4f27f50af3050d625a43622f8e004eed",
        "80d99024bcffccb1109c158b318eb3262beeb53930edcf5c513bb342ee09e6aa",
        "8269225d993f4b307e02374f2abfd3854fc8c0b019978dbc8f4553b763e8b52b",
        "82f7588c4db12d63fe7d5e2a204dcbd0674a57ac284c04fe196052a0f188a8c8",
        "8350b0b87334802bf437d8073829c537225eabc153095970247c4764e3eba5d2",
        "85728e8125e1c99f83547ccd42fdde0e8baa3c175389bc729121551dcbefe601",
        "8647cd82b6ffe4a543b0e3b38e2a9fb928b92f221f24c947f63bb4b6985c4a1c",
        "87c763cd2d60e3828f78aea0a5e4e7f998ecc01ed0796c1b52f959cb7329e7c0",
        "896678bf1d199f63110c54651aab212226f46650f843f0c9d7ea4351f89d8693",
        "89ce3f7740914e0a28e8bf87773e55e254c506fd45f35930a8e1534a4fd90296",
        "8bfd031c421e6bc1a0ecd2539a07394134e027f780acfc2cad47dceffd9fe7bf",
        "8d33b59a01347c705233c36ae9fbcc4b0a3725124d8b78499d1596136d6c6101",
        "8f7ba797f54adc09ff69852553755e74687574011ae9498bc85ad152bf287a3f",
        "9059042d5eccd6b91e126e4f265a62ca7c5b731e4ee5669f9cbd84484707ee22",
        "927e7c89a1cf8b7fe83160ecd27589da1d7907a91d2b549f39d28304937b3baa",
        "971a82a24e34d35b7d720c2876245bd53aa6e2e1aa13201dd8fc59d8156151a9",
        "97d89567701c71f2964efb7ae05ce8979f35b0f50347587cbf411e64084cccd9",
        "98c8890cedb29dbb9c2052bdcff7a35e4172241fc072146348960593386d1be8",
        "9982a2d5e0ef4fe7ea447f38318e741be8ec41f249c431fcc46da7610e8de1b8",
        "9a1fc159075eced3f4ed4fd94b6be694719e2c6d77882c4fc4fb9b34ce37232c",
        "9a5c18d885838aaa6320f2bf9058a52a5c30217f6559e4fb8a4de11b892e51d9",
        "9aa5693171fb008a608d975b62e666635406ca04239ed92533eb18197442f4d4",
        "9d9d8045030ef22e989749f6b01335293d396849ff4391cf79f3abf00a2bebdb",
        "9f5c0a722ffeed0a7d4baacf767a716c2b1ab077a005e97584080e5af0df3a01",
        "a02d1e881eac9a436caab199c5076f24149062507df89b57974588372d2791cd",
        "a1c448ced7ed5f2d88f56e66add5678be2144115e7a88d03d3c92ca884a5c46d",
        "a1d9152ef9d38ce8518288b3e874dde9a0aacaac8c111326b0f3ff6846681189",
        "a26e1f3bc50584a19b9de75128ecf6773e014f9779363278de2452dcd6490453",
        "a2e99d8f261d3ad7e1ab44f6863996947c860bd077c13d90a1a03fee0312565d",
        "a3dc03b7812beba7da6f9c0c8799dcad2e835869404510302c07a5f7a132d046",
        "a90961c159a2923dac8e7940621edccce05f69e615f1cccabd44fbf134825445",
        "a9184dd1c39094257a072f828d8a1edb86fac5f2a8317b5fbf6050fa6d4995d2",
        "ab1ab395fe3f223705f3e04256831b674ad4d5f43cdda8164d7eddf4e91c6a01",
        "b0aeea04208b7ee102373b2b39327acf1c1ce37b7a2cb0993ab570aca92ccaef",
        "b4702f900d1df7b2dacf7c985e712adde7e56a5441b685a8ef690df9508a73c7",
        "b61a5c5b568522367421c2a8b8cf229078d2a9827ba249d4bfc678a71c1fef72",
        "ba53ea40072a4ce1b209d51a243199b602cd61c1556e07c40f9021281b82c4c9",
        "bd5574074427bf0882f6f229ac1a3d0801d418204aca6e38329f55d149196106",
        "bda7d73b76624f1711424e4539b1638fb9f291509af6e4bbaca35ee06620c3e0",
        "bf54ba82e3acf0a8d801946082049b0e482b49959050118d347a0dc88313ca5d",
        "bfce22b0e14dd865ca1e0d48be289623f790a98d082999b34189502ff80a9293",
        "bfeb40599bfb05f22d79850f4c234ccc4a0ccb61a043e0934933cd7a604ea351",
        "c0153eb96c17b56940e89db3133b8ccf357c6ddbd832b611db5e0bce176b721d",
        "c016b085794e0423df0396fb9573ab0cd4b04135f588f4b465d39494b8f50863",
        "c0502590458f996605f01fc3e251c7f15ed8a2534bb823df4fb528cbfd01d816",
        "c45df6bb6e37cf47410d18a9d059d0c877477becabeece66e73901f1503ca727",
        "c4c4a60dae070437f1ff0106a007f59862818b6299463092597af335ddb82133",
        "c5bc6670ffc5450ea49f77e31cae7bc2c0a791611aca0491578f2fbbc7585ae3",
        "c68a65e6f02919bd47a7f6e9a3faf4a8c1fcae75d0825b45d9c2acdb763aff81",
        "c75bc036fcad8ca4c9cfaaeeb307dfb49270afc6bcb1508752bcdf313b9ee11e",
        "c8d4017d4215eefad4d221b8b9cabd5ffbc38a5c1c2a04db54d0c15a58d73660",
        "cdc113b03e21515618181aabc873e493584cfaf65de0f472f3391be3ded32e0c",
        "cfba22607b0455cb68f3771ab271dee3cefa3a07a70fb4d43739722651f574c1",
        "d1ad0ffe04181a64643e29e349266ef4fc8d185652339654287f0386bf5b0b41",
        "d262841680de741735162f725353cc21ef67876bafe32efee147a3d5a0f36fce",
        "d31297042983582a094baafe9b83d7205959de2f9c429378bda7815923aad63b",
        "d4bb05ad677d141296dadec69c31d406d1200ed4766e7ddfdcabd5b45739cd16",
        "d5e63d453b8b782c4770f71b8f528bd4c6dcc41a9be16dcef6518b2a8deec9a8",
        "d638bc685db2d40a4eabc959738c0b0f04d7e048087e4e7d79e267991baf84f0",
        "d99c1197c4c5ac0bce678e8fe5410950c832e17c9f7410865fd2e3889090d005",
        "da748574f7d35e0a277d276c26395b11ad372f8965fc5500a14a1a9a5244afdc",
        "dabc11c7626d140584c69e2a877910656604a22767f2f83cbe9d962f99158baa",
        "daf595aa13c9f4bfd930d3579759bbe12d632ee352a49bed67471937cb488575",
        "db597db1cecbfe54450fa0be67c2d645ceac2212db05c728858b146961f49ea3",
        "dc0eb934224f160dea04f6e71fec9f58a7795de3a8ad4d1dbaf2d4bbd3f66fdb",
        "dd52de5b50fe4e297066cc47448c8df85de2cbfb19bd91a8338a516399d3d250",
        "df58196f971916a328c95fb40bdab839053ea453e7a7adaf02dc88bed2a63489",
        "e09d633d3b4039c8daa827c878c1e1392e627ba7ab606d790585552207f03813",
        "e2280eda5b4ebfd988a99cb84bed9eea6645f6dfb99e7439bd2121f6de46a787",
        "e2b5393f59c4c6afbf7d662643e78cdce842a99ddaa9d9100a2d63b72ac9dd13",
        "e303e209682d2f22fd64a75e4a77d3d950671c1130788840cb6fa300da3f9daf",
        "e31559353b0edfeff269672dc3a484e60c427e554d5b54c571b2d33c1d89e2b3",
        "e3b2fe3a7a67ef32b304f6c135b8c6e69e8b9b1b8d11144ed33e164da7203496",
        "e3eb1f87cebd241255c67ef33727b29d85e2bd9e1efe1671c584db06f4df6a6e",
        "e5c568ecd37e22a50a0c8e2fc63cd2b9f9adb8b43a100b462135c2a261eb6e12",
        "e635329855034b4d435b8b1a41d14677895bee626cfc6158783bfee71abc79b5",
        "e75883268c7e376c0dd7a730b8056e1796f8efec121e7cd7cae99f61294e61c3",
        "eb5e64453a936e6eb34a728f4d72c5e3cd83a44ce832cd8d8ba551b87bde1099",
        "eb6de055a41794d587af9089da5c7b6113474221572ddbf58c21752d92dc422d",
        "edbb262aeccb6f1145e735d427e3c07d74fdca8843e138bc53a647bc444b7d90",
        "ee46d15faf8160e2289c0b6660de6fcd2c25a80f2d2c50b3f70bc78339b4a52b",
        "f037551c416b7782e2351df49a18b50acec5b4ef358701a1d7915aa97c9af481",
        "f12b4d07269097a34fee893ed751673f79456941a541a50b2b54e7215a8a38eb",
        "f1bc436b8fd2cdc55c9208c368241b877fdebeb339fab3fa3b76f7f76c135ed4",
        "f2ae41945edf9ba34264ca2abb823519cd41003854a24190629053b8c05c1043",
        "f3aa5c97c7927028185d14acaecdd0f1f567a9c85973816042130c904dcdf966",
        "f5d9051f7cb5ab32e54d471063c110aeca59b13e927e324613874fa124d82476",
        "f6d0b56bbc173ca4e6a19eab84e130d48e4d87ddebba6a2160e023ebb5c83be3",
        "f7572ad50a30ab2f145da32bf1d55cc172df304ec94676f09d8be31857d98268",
        "f77914d2d284120998e7fec0aa1d5828abcf0ac9d8476c443dbe614e46fa8bc0",
        "faa40c5268a08579c6fd28b39e8cb495c43e40f16bf545257367176b36a8c32f",
    ]),
];

// Principal addresses that need conversion to account IDs
pub const PATTERN_PRINCIPALS: &[(&str, &str)] = &[
    ("David the Gnome", "aiuxi-qgbbo-2bls4-7ac4x-suec5-bo6mm-zq6yh-asr25-iug6d-s7csv-jae"),
    ("David Fisher WTN", "cld52-vm6st-5ulwe-yperp-iwvft-gqt7a-jrbpm-pkdcl-yszk3-zyxvb-wae"),
];

// Get all pattern addresses as a map: address -> name
pub fn get_all_pattern_addresses() -> HashMap<String, String> {
    let mut addresses = HashMap::new();
    
    // Add hex addresses
    for (name, addrs) in PATTERN_SEED_ADDRESSES {
        for addr in *addrs {
            addresses.insert(addr.to_string(), name.to_string());
        }
    }
    
    // Add converted principal addresses (default subaccount)
    for (name, principal_str) in PATTERN_PRINCIPALS {
        if let Ok(principal) = Principal::from_text(principal_str) {
            let account_id = principal_to_account_id(&principal, None);
            let hex = hex::encode(account_id);
            addresses.insert(hex, name.to_string());
        }
    }
    
    addresses
}

// Get just the addresses as a vector for easy iteration
pub fn get_pattern_address_list() -> Vec<String> {
    let mut addresses = Vec::new();
    
    // Add hex addresses
    for (_, addrs) in PATTERN_SEED_ADDRESSES {
        for addr in *addrs {
            addresses.push(addr.to_string());
        }
    }
    
    // Add converted principal addresses
    for (_, principal_str) in PATTERN_PRINCIPALS {
        if let Ok(principal) = Principal::from_text(principal_str) {
            let account_id = principal_to_account_id(&principal, None);
            let hex = hex::encode(account_id);
            addresses.push(hex);
        }
    }
    
    addresses
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_addresses() {
        let addresses = get_all_pattern_addresses();
        
        // Should have 17 seed addresses + 2 converted principals = 19 total
        assert!(addresses.len() >= 19);
        
        // Check some known addresses exist
        assert!(addresses.contains_key("14e7d1ac542c0bce0be9953ce0ee8e99ea6d4cb3756db2ad1efdaeabc6bd24f5"));
        assert!(addresses.contains_key("55d6c8c9bf841d721785e422130a385f13e71d8b5431c65b8be6d2b3a03d0c28"));
    }
    
    #[test]
    fn test_principal_conversion() {
        let addresses = get_pattern_address_list();
        
        // Should have all 19 addresses
        assert_eq!(addresses.len(), 19);
        
        // All should be valid hex strings
        for addr in &addresses {
            assert_eq!(addr.len(), 64); // 32 bytes = 64 hex chars
            assert!(addr.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }
}