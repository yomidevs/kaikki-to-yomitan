import * as TermBank from './node_modules/yomichan-dict-builder/src/types/yomitan/termbank';
import * as TagBank from './node_modules/yomichan-dict-builder/src/types/yomitan/tagbank';
import * as TermBankMeta from './node_modules/yomichan-dict-builder/src/types/yomitan/termbankmeta';

declare global {
    // 3-tidy-up.js types:

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
        examples?: Example[];
        glosses?: Glosses;
        raw_glosses?: Glosses;
        raw_gloss?: Glosses;
        tags?: string[];
        raw_tags?: string[];
        form_of?: FormOf[];
    }

    type Example = {
        text?: string;
        type?: "example" | "quotation";
        english?: string;
        roman?: string;
    }

    type Glosses = string | string[];
    
    type FormOf = {
        word?: string;
    }

    type GlossTree = Map<string, GlossBranch> ;

    type GlossBranch = Map<string, GlossTwig> & {
        get(key: '_tags'): string[] | undefined;
        set(key: '_tags', value: string[]): GlossBranch;
        get(key: '_examples'): Example[] | undefined;
        set(key: '_examples', value: Example[]): GlossBranch;
    } ;

    type GlossTwig = Map<string, GlossTwig>;
      
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
        glosses: TermBank.DetailedDefinition[],
        tags: string[],
        examples: Example[],
    }
    
    type Lemma = string;
    type Form = string;
    type PoS = string;
    type FormsMap = Map<Lemma, Map<Form, Map<PoS, string[]>>>;
    type AutomatedForms = Map<Lemma, Map<Form, Map<PoS, Set<string>|string[]>>>;

    type NestedObject = {
        [key: string]: NestedObject | any;
    }

    // 4-make-yomitan.js types:
    type MakeYomitanEnv = {
        source_iso: string,
        target_iso: string,
        DEBUG_WORD?: string,
        DICT_NAME: string,
        tidy_folder: string,
        temp_folder: string,
    }

    type CondensedFormEntries = [string, string, [string, string[]][]][];

    type WhitelistedTag = [
        shortTag: string,
        category: string,
        sortOrder: number,
        longTag: string | string[], // if array, first element will be used, others are aliases
        popularityScore: number,
    ]
}

export {
    TermBank,
    TagBank,
    TermBankMeta
}