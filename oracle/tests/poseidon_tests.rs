#[cfg(test)]
mod tests {
    use algebra::{pasta::Fp, fields::PrimeField, BigInteger256, UniformRand};
    use oracle::poseidon::Sponge; // needed for ::new() sponge

    use oracle::poseidon::ArithmeticSponge as Poseidon3Wires;
    use oracle::poseidon::PlonkSpongeConstants as Constants3Wires;
    use oracle::pasta::fp as Parameters3Wires;

    use oracle::poseidon_5_wires::ArithmeticSponge as Poseidon5Wires;
    use oracle::poseidon_5_wires::PlonkSpongeConstants as Constants5Wires;
    use oracle::pasta::fp5 as Parameters5Wires;

    fn _rand_fields(n: u8) {
        let rng = &mut rand::thread_rng();
        for _i in 0..n {
            let fe = Fp::rand(rng);
            println!("{:?}", fe.into_repr());
        }
    }

    #[test]
    fn poseidon_3_wires() {
        macro_rules! assert_poseidon_3_wires_eq {
            ($input:expr, $target:expr) => {
                let mut s = Poseidon3Wires::<Fp, Constants3Wires>::new();
                s.absorb(&Parameters3Wires::params(), $input);
                let output = s.squeeze(&Parameters3Wires::params());
                assert_eq!(output, $target, "\n output: {:?}\n target: {:?}", output.into_repr(), $target.into_repr());
            };
        }

        // _rand_fields(0);
        assert_poseidon_3_wires_eq!(
            &[
            ],
            Fp::from_repr(BigInteger256([17114291637813588507, 14335107542818720711, 1320934316380316157, 1722173086297925183]))
        );

        // _rand_fields(1);
        assert_poseidon_3_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([11416295947058400506, 3360729831846485862, 12146560982654972456, 2987985415332862884]))
            ],
            Fp::from_repr(BigInteger256([871590621865441384, 15942464099191336363, 2836661416333151733, 11819778491522761]))
        );

        // _rand_fields(2);
        assert_poseidon_3_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([16049149342757733248, 17845879034270049224, 6274988087599189421, 3891307270444217155])),
                Fp::from_repr(BigInteger256([9941995706707671113, 236362462947459140, 17033003259035381397, 4098833191871625741]))
            ],
            Fp::from_repr(BigInteger256([17256859529285183666, 10562454737368249340, 16653501986100235558, 1613229473904780795]))
        );

        // _rand_fields(3);
        assert_poseidon_3_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([16802949773563312590, 13786671686687654025, 6327949131269833714, 2206832697832183571])),
                Fp::from_repr(BigInteger256([18422989176992908572, 7121908340714489421, 15983151711675082713, 2047309793776126211])),
                Fp::from_repr(BigInteger256([10656504003679202293, 5033073342697291414, 15641563258223497348, 2549024716872047224]))
            ],
            Fp::from_repr(BigInteger256([4610990272905062813, 1786831480172390544, 12827185513759772316, 1463055697820942106]))
        );

        // _rand_fields(6);
        assert_poseidon_3_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([13568896335663078044, 12780551435489493364, 7939944734757335782, 2716817606766379733])),
                Fp::from_repr(BigInteger256([8340509593943796912, 14326728421072412984, 1939214290157533341, 248823904156563876])),
                Fp::from_repr(BigInteger256([18138459445226614284, 7569000930215382240, 12226032416704596818, 754852930030810284])),
                Fp::from_repr(BigInteger256([11813108562003481058, 3775716673546104688, 7004407702567408918, 2198318152235466722])),
                Fp::from_repr(BigInteger256([9752122577441799495, 2743141496725547769, 8526535807986851558, 1154473298561249145])),
                Fp::from_repr(BigInteger256([12335717698867852470, 17616685850532508842, 8342889821739786893, 2726231867163795098]))
            ],
            Fp::from_repr(BigInteger256([2534358780431475408, 3747832072933808141, 2500060454948506474, 2342403740596596240]))
        );
    }

    #[test]
    fn poseidon_5_wires() {
        macro_rules! assert_poseidon_5_wires_eq {
            ($input:expr, $target:expr) => {
                let mut s = Poseidon5Wires::<Fp, Constants5Wires>::new();
                s.absorb(&Parameters5Wires::params(), $input);
                let output = s.squeeze(&Parameters5Wires::params());
                assert_eq!(output, $target, "\n output: {:?}\n target: {:?}", output.into_repr(), $target.into_repr());
            };
        }

        // _rand_fields(0);
        assert_poseidon_5_wires_eq!(
            &[
            ],
            Fp::from_repr(BigInteger256([15786247561146094417, 17389383855131330079, 13754064673928145000, 2065613552271182841]))
        );

        // _rand_fields(1);
        assert_poseidon_5_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([925605326051629702, 9450022185177868805, 3430781963795317176, 2120098912251973017]))
            ],
            Fp::from_repr(BigInteger256([1118209274952071971, 17394742092044022630, 12874178937932366101, 3658930256824255443]))
        );

        // _rand_fields(2);
        assert_poseidon_5_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([4872213112846934187, 15221974649365942201, 4177652558587823268, 1324361518338458527])),
                Fp::from_repr(BigInteger256([10368205141323064185, 9471328583611422132, 12997197966961952901, 3290733940621514661]))
            ],
            Fp::from_repr(BigInteger256([17050973263717943469, 6148223662119308948, 4211607956951971921, 2074712008335412085]))
        );

        // _rand_fields(4);
        assert_poseidon_5_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([7832849012654337787, 4963068119957452774, 10773086124514989319, 1683727612549340848])),
                Fp::from_repr(BigInteger256([3569008656860171438, 10394421784622027030, 196192141273432503, 1248957759478765405])),
                Fp::from_repr(BigInteger256([9522737303355578738, 572132462899615385, 13566429773365192181, 121306779591653499])),
                Fp::from_repr(BigInteger256([13250259935835462717, 4425586510556471497, 14507184955230611679, 2566418502016358110]))
            ],
            Fp::from_repr(BigInteger256([5177184197044066934, 3278668611512352244, 8179179603823460349, 2573133080118749581]))
        );

        // _rand_fields(5);
        assert_poseidon_5_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([17910451947845015148, 5322223719857525348, 10480894361828395044, 34781755494926625])),
                Fp::from_repr(BigInteger256([6570939701805895370, 4169423915667089544, 2366634926126932666, 1804659639444390640])),
                Fp::from_repr(BigInteger256([13670464873640336259, 14938327700162099274, 9664883370546456952, 2153565343801502671])),
                Fp::from_repr(BigInteger256([6187547161975656466, 12648383547735143102, 15485540615689340699, 417108511095786061])),
                Fp::from_repr(BigInteger256([3554897497035940734, 1047125997069612643, 8351564331993121170, 2878650169515721164]))
            ],
            Fp::from_repr(BigInteger256([6314601368399953808, 1127070340883698633, 3075801212459179947, 3772592231561709761]))
        );

        // _rand_fields(6);
        assert_poseidon_5_wires_eq!(
            &[
                Fp::from_repr(BigInteger256([13179872908007675812, 15426428840987667748, 15925112389472812618, 1172338616269137102])),
                Fp::from_repr(BigInteger256([9811926356385353149, 16140323422473131507, 1062272508702625050, 1217048734747816216])),
                Fp::from_repr(BigInteger256([9487959623437049412, 8184175053892911879, 12241988285373791715, 528401480102984021])),
                Fp::from_repr(BigInteger256([2797989853748670076, 10357979140364496699, 12883675067488813586, 2675529708005952482])),
                Fp::from_repr(BigInteger256([8051500605615959931, 13944994468851713843, 9308072337342366951, 3594361030023669619])),
                Fp::from_repr(BigInteger256([6680331634300327182, 6761417420987938685, 10683832798558320757, 2470756527121432589]))
            ],
            Fp::from_repr(BigInteger256([8569852085098762347, 6223602240796629432, 17589537382252822232, 3103610489425777467]))
        );
    }
}
