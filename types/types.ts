declare global {
    type TidyEnv = {
        source_iso: string,
        target_iso: string,
        kaikki_file: string,
        tidy_folder: string,
    }

     type KaikkiLine = {
        head_templates?: HeadTemplate[];
        word?: string;
        pos?: string;
        sounds?: Sound[];  
        forms?: FormInfo[];
        senses?: KaikkiSense[];
    }

    type HeadTemplate = {
        name?: string;
        args?: string[];
    }

    type Sound = {
        ipa?: string|string[];
        tags?: string[];
        note?: string;
    }

    type FormInfo = {
        form?: string;
        tags?: string[];
    }

    type KaikkiSense = {
        glosses?: Glosses;
        raw_glosses?: Glosses;
        raw_gloss?: Glosses;
        tags?: string[];
        raw_tags?: string[];
        form_of?: FormOf[];
    }

    type Glosses = string | string[];
    
    type FormOf = {
        word?: string;
    }

    type GlossTree = Map<string, GlossTree> & {
        get(key: '_tags'): string[] | undefined;
        set(key: '_tags', value: string[]): GlossTree;
    };
      
    type TidySense = Omit<KaikkiSense, 'tags'> & {
        tags: string[];
        glossesArray: string[];
    }

    type LemmaDict = {
        [word: string]: {
            [reading: string]: {
                [pos: string]: LemmaInfo
            }
        }
    }

    type LemmaInfo = {
        ipa: IpaInfo[],
        senses: SenseInfo[],
    }

    type IpaInfo = {
        ipa: string,
        tags: string[],
    }

    type SenseInfo = {
        glosses: YomitanGloss[],
        tags: string[],
    }

    type YomitanGloss = string | StructuredGloss
    
    type StructuredGloss = {
        type: "structured-content",
        content: string | StructuredContent[],
    }

    type StructuredContent = {
        tag: string,
        data: string,
        content: StructuredContent,
    }

    type Lemma = string;
    type Form = string;
    type PoS = string;
    type FormsMap = Map<Lemma, Map<Form, Map<PoS, string[]>>>;
    type AutomatedForms = Map<Lemma, Map<Form, Map<PoS, Set<string>|string[]>>>;

    type NestedObject = {
        [key: string]: NestedObject | any;
    }
}

export {} // This is needed to make this file a module