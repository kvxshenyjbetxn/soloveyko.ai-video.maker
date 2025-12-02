import json
import re

text_data = """
neural	Afrikaans, South Africa	af-ZA	ai3-af-ZA-Kungawo, ai3-af-ZA-Sura
neural	Amharic (Ethiopia)	am-ET	ai3-am-ET-Tamru, ai3-am-ET-Mazaa
neural	Arabic (United Arab Emirates)	ar-AE	ai3-ar-AE-Hamiz, ai3-ar-AE-Paree

ai1-ar-AE-Nura, ai1-ar-AE-Hamza
neural	Arabic (Bahrain)	ar-BH	ai3-ar-BH-Ali, ai3-ar-BH-Pareesha
neural	Arabic (Algeria)	ar-DZ	ai3-ar-DZ-Samia, ai3-ar-DZ-Khalil
neural	Arabic (Iraq)	ar-IQ	ai3-ar-IQ-Vaneeza, ai3-ar-IQ-Ganief
neural	Arabic (Jordan)	ar-JO	ai3-ar-JO-Ebrahim, ai3-ar-JO-Saabiha
neural	Arabic (Kuwait)	ar-KW	ai3-ar-KW-Fyaz, ai3-ar-KW-Naaz
neural	Arabic (Lebanon)	ar-LB	ai3-ar-LB-Delkash, ai3-ar-LB-Hamees
neural	Arabic (Libya)	ar-LY	ai3-ar-LY-Ieesha, ai3-ar-LY-Mahfuj
neural	Arabic (Morocco)	ar-MA	ai3-ar-MA-Lajin, ai3-ar-MA-Ozza
neural	Arabic (Oman)	ar-OM	ai3-ar-OM-Adnan, ai3-ar-OM-Zulima
neural	Arabic (Qatar)	ar-QA	ai3-ar-QA-Nabeel, ai3-ar-QA-Azma
neural	Arabic (Saudi Arabia)	ar-SA	ai3-ar-SA-Hamed, ai3-ar-SA-Zariyah
neural	Arabic (Syria)	ar-SY	ai3-ar-SY-Gulbar, ai3-ar-SY-Sumiya
neural	Arabic (Tunisia)	ar-TN	ai3-ar-TN-Hadeeqa, ai3-ar-TN-Hadeeqa
neural	Arabic (Yemen)	ar-YE	ai3-ar-YE-Wabisa, ai3-ar-YE-Parwaz
neural	Arabic	arb	ai2-ar-XA-Nadir, ai2-ar-XA-Sana, ai2-ar-XA-Iman, ai2-ar-XA-Fatima

ai3-ar-XA-Shakir, ai3-ar-XA-Salma
neural	Assamese (India)	as-IN	ai3-as-IN-Tiyasha, ai3-as-IN-Mondip
neural	Azerbaijani (Azerbaijan)	az-AZ	ai3-az-AZ-Leyla, ai3-az-AZ-Farid
neural	Bulgarian, Bulgaria	bg-BG	ai3-bg-BG-Gergana, ai3-bg-BG-Boyan
neural	Bangla (Bangladesh)	bn-BD	ai3-bn-BD-Devyani, ai3-bn-BD-Omar
neural	Bengali (India)	bn-IN	ai2-bn-IN-Binod, ai2-bn-IN-Charu

ai3-bn-IN-Koel, ai3-bn-IN-Neel
neural	Bosnian (Bosnia and Herzegovina)	bs-BA	ai3-bs-BA-Behrem, ai3-bs-BA-Farid
neural	Catalan, Spain	ca-ES	ai3-ca-ES-Enric, ai3-ca-ES-Alba, ai3-ca-ES-Joana

ai1-ca-ES-Estel
neural	Chinese, Mandarin	cmn-CN	ai3-cmn-CN-Xiomara, ai3-cmn-CN-Yunye, ai3-cmn-CN-Xariyah, ai3-cmn-CN-Yunyang, ai3-cmn-CN-Carissa, ai3-cmn-CN-Xiaoxiao, ai3-cmn-CN-Xylia, ai3-cmn-CN-Xiaoyou, ai3-cmn-CN-Xander, ai3-cmn-CN-Mingxia, ai3-cmn-CN-Ayaka, ai3-cmn-CN-Xiaosheng, ai3-cmn-CN-Xiulin, ai3-cmn-CN-Yichen, ai3-cmn-CN-Junfeng, ai3-cmn-CN-Mei, ai3-cmn-CN-Yunze, ai3-cmn-CN-Fang, ai3-cmn-CN-Zihan, ai3-cmn-CN-Jiahui, ai3-cmn-CN-Yuhang, ai3-cmn-CN-ChangV2

ai2-cmn-CN-Claire, ai2-cmn-CN-Yao, ai2-cmn-CN-Sue, ai2-cmn-CN-Vincent

ai1-cmn-CN-Shiyun
neural	Chinese, Mandarin (Taiwan)	cmn-TW	ai2-cmn-TW-Ting, ai2-cmn-TW-Bao, ai2-cmn-TW-Qiang

ai3-cmn-TW-HsiaoYu, ai3-cmn-TW-Sachihiro, ai3-cmn-TW-HsiaoChen
neural	Czech (Czech Republic)	cs-CZ	ai2-cs-CZ-Eliska

ai3-cs-CZ-Vlasta, ai3-cs-CZ-Antonin
neural	Welsh	cy-GB	ai3-cy-GB-Gareth, ai3-cy-GB-Catrin
neural	Danish (Denmark)	da-DK	ai3-da-DK-Christel, ai3-da-DK-Jeppe

ai2-da-DK-Johan, ai2-da-DK-Signe, ai2-da-DK-Abbie, ai2-da-DK-Julie

ai1-da-DK-Esther
neural	German, Austria	de-AT	ai3-de-AT-Ingrid, ai3-de-AT-Jonas

ai1-de-AT-Melissa
neural	German, Switzerland	de-CH	ai3-de-CH-Noah, ai3-de-CH-Anja
neural	German	de-DE	ai4-de-DE-Paul, ai4-de-DE-Anja, ai4-de-DE-Gabriele

ai2-de-DE-Patrick, ai2-de-DE-Pia, ai2-de-DE-Mona, ai2-de-DE-Dustin, ai2-de-DE-Fabienne, ai2-de-DE-Thomas

ai3-de-DE-Katja, ai3-de-DE-Conrad, ai3-de-DE-Johanna, ai3-de-DE-Kasper, ai3-de-DE-Schmidt, ai3-de-DE-Galliena, ai3-de-DE-Marlene, ai3-de-DE-Ermanno, ai3-de-DE-Rodriguez, ai3-de-DE-Rheinbeck, ai3-de-DE-Kerryl, ai3-de-DE-Marie, ai3-de-DE-Brunon, ai3-de-DE-Yettie, ai3-de-DE-Maja, ai3-de-DE-AmaliaV2

ai1-de-DE-Fiona, ai1-de-DE-Stefan

ai5-de-DE-Mathilda
neural	Greek (Greece)	el-GR	ai2-el-GR-Anastasia

ai3-el-GR-Athina, ai3-el-GR-Topher
neural	English, Australian	en-AU	ai1-Olivia

ai3-Natasha, ai3-William, ai3-en-AU-Jacob, ai3-en-AU-Stella, ai3-en-AU-Joshua, ai3-en-AU-Emma, ai3-en-AU-Maddison, ai3-en-AU-Edward, ai3-en-AU-Sonny, ai3-en-AU-Sienna, ai3-en-AU-Claire, ai3-en-AU-Daisy, ai3-en-AU-Grace, ai3-en-AU-Logan

ai2-Oliver, ai2-Matilda, ai2-Harry, ai2-Amelia, ai2-en-AU-Amelia2, ai2-en-AU-Matilda2, ai2-en-AU-Viaan, ai2-en-AU-Liya, ai2-en-AU-Oliver2, ai2-en-AU-Harry2, ai2-en-AU-Amanda

ai4-en-AU-Amaya, ai4-en-AU-Nelson
neural	English, Canada	en-CA	ai3-en-CA-Clara, ai3-en-CA-Liam
neural	English, British	en-GB	ai4-Harry, ai4-Elizabeth, ai4-Niamh

ai1-Amy, ai1-Emma, ai1-Brian, ai1-en-GB-George

ai3-Libby, ai3-Ryan, ai3-Mia, ai3-en-GB-Maria, ai3-en-GB-Lyra, ai3-en-GB-Rose, ai3-en-GB-Dylan, ai3-en-GB-Jasper, ai3-en-GB-David, ai3-en-GB-Hollie, ai3-en-GB-Thomas, ai3-en-GB-Alexander, ai3-en-GB-Hudson, ai3-en-GB-Hannah, ai3-en-GB-Bella

ai2-Freddie, ai2-William, ai2-Jessica, ai2-Emily, ai2-Victoria, ai2-en-GB-Bella2, ai2-en-GB-Lily2, ai2-en-GB-Maya, ai2-en-GB-Victoria2, ai2-en-GB-Calvin, ai2-en-GB-Jessica2, ai2-en-GB-Jax, ai2-en-GB-Erin, ai2-en-GB-Zayn, ai2-en-GB-Dexter, ai2-en-GB-William2, ai2-en-GB-Lucy, ai2-en-GB-Freddie2
neural	English, Hong Kong	en-HK	ai3-en-HK-Rachel, ai3-en-HK-Zach
neural	English, Ireland	en-IE	ai3-en-IE-Connor, ai3-en-IE-Emily

ai1-en-IE-Aoife
neural	English, Indian	en-IN	ai2-en-IN-Rohan, ai2-en-IN-Luv, ai2-en-IN-Tanvi, ai2-en-IN-Alisha, ai2-en-IN-Tanvi2, ai2-en-IN-Alisha2, ai2-en-IN-Rohan2, ai2-en-IN-Luv2

ai3-en-IN-Neerja, ai3-en-IN-Prabhas, ai3-en-IN-Kavita, ai3-en-IN-Ankita, ai3-en-IN-Megha, ai3-en-IN-Karan

ai1-en-IN-Kavya
neural	English, Kenya	en-KE	ai3-en-KE-Reth, ai3-en-KE-Almasi
neural	English, Nigeria	en-NG	ai3-en-NG-Adaeze, ai3-en-NG-Gicicio
neural	English, New Zealand	en-NZ	ai3-en-NZ-Sebastian, ai3-en-NZ-Becca

ai1-Amelia
neural	English, Philippines	en-PH	ai3-en-PH-Luwalhati, ai3-en-PH-Magiting
neural	English, Singapore	en-SG	ai3-en-SG-Richard, ai3-en-SG-Juan

ai1-en-SG-Alyssa
neural	English, Tanzania	en-TZ	ai3-en-TZ-Vinza, ai3-en-TZ-Neema
neural	English, US	en-US	ai2-Stacy, ai2-John2, ai2-Robert2, ai2-Scott, ai2-Scarlet, ai2-Jerry, ai2-Kathy, ai2-Isabella, ai2-Nikola, ai2-Katie, ai2-en-US-Jaxson2, ai2-en-US-Katie2, ai2-en-US-Soren, ai2-en-US-Isabella2, ai2-John, ai2-Robert, ai2-en-US-Stacy2, ai2-en-US-Jerry2, ai2-en-US-Milo, ai2-en-US-Kathy2, ai2-en-US-Maeve, ai2-en-US-Aurora, ai2-en-US-Scarlet2

ai3-Nova, ai3-Jony, ai3-Olive, ai3-Vienna, ai3-Emily, ai3-Addyson, ai3-Evan, ai3-Jenny, ai3-Taylor, ai3-Kailey, ai3-Kingsley, ai3-Jason, ai3-Gary, ai3-Aria, ai3-en-US-Kaiya, ai3-en-US-Ashley, ai3-en-US-Alexander, ai3-en-US-Joshua, ai3-en-US-Jayden, ai3-en-US-Sage, ai3-en-US-Austin, ai3-en-US-Lucas, ai3-en-US-Madison, ai3-en-US-GraysonV2, ai3-en-US-Logan, ai3-en-US-BrysonV2, ai3-en-US-EleanorV2

ai1-Kevin, ai1-Joanna, ai1-Justin, ai1-Kimberly, ai1-Kendra, ai1-Matthew, ai1-Joey, ai1-Salli, ai1-Ivy, ai1-en-US-Jack, ai1-en-US-Luna, ai1-en-US-Joseph, ai1-en-US-Evelyn

ai4-Samantha, ai4-Doris, ai4-Edward, ai4-Amanda, ai4-Roger, ai4-Ronald, ai4-Sophia, ai4-en-US-Ariana

ai6-en-US-Voice5, ai6-en-US-Voice1, ai6-en-US-Voice4, ai6-en-US-Voice2, ai6-en-US-Voice3

ai13-en-US-HashCode36, ai13-en-US-HashCode37, ai13-en-US-HashCode5, ai13-en-US-HashCode10, ai13-en-US-HashCode19, ai13-en-US-HashCode21, ai13-en-US-HashCode, ai13-en-US-HashCode17, ai13-en-US-HashCode22, ai13-en-US-HashCode23, ai13-en-US-HashCode29, ai13-en-US-HashCode3, ai13-en-US-HashCode15, ai13-en-US-HashCode41, ai13-en-US-HashCode30, ai13-en-US-HashCode9, ai13-en-US-HashCode16, ai13-en-US-HashCode42, ai13-en-US-HashCode2, ai13-en-US-HashCode7, ai13-en-US-HashCode18, ai13-en-US-HashCode25, ai13-en-US-HashCode39, ai13-en-US-HashCode47, ai13-en-US-HashCode35, ai13-en-US-HashCode44, ai13-en-US-HashCode43, ai13-en-US-HashCode38, ai13-en-US-HashCode46, ai13-en-US-HashCode31, ai13-en-US-HashCode4, ai13-en-US-HashCode26, ai13-en-US-HashCode6, ai13-en-US-HashCode11, ai13-en-US-HashCode8, ai13-en-US-HashCode20, ai13-en-US-HashCode24, ai13-en-US-HashCode40, ai13-en-US-HashCode48, ai13-en-US-HashCode34, ai13-en-US-HashCode12, ai13-en-US-HashCode45, ai13-en-US-HashCode14, ai13-en-US-HashCode13, ai13-en-US-HashCode28, ai13-en-US-HashCode27, ai13-en-US-HashCode49, ai13-en-US-HashCode32, ai13-en-US-HashCode33, ai13-en-US-HashCode51, ai13-en-US-HashCode53, ai13-en-US-HashCode52
neural	English, South Africa	en-ZA	ai3-en-ZA-Amara, ai3-en-ZA-Evans

ai1-Mandisa
neural	Spanish, Argentina	es-AR	ai3-es-AR-Hernan, ai3-es-AR-Malen
neural	Spanish, Bolivia	es-BO	ai3-es-BO-Eduardo, ai3-es-BO-Labanya
neural	Spanish, Chile	es-CL	ai3-es-CL-Eliana, ai3-es-CL-Vicente
neural	Spanish, Colombia	es-CO	ai3-es-CO-Brandon, ai3-es-CO-Luciana
neural	Spanish, Costa Rica	es-CR	ai3-es-CR-Antonio, ai3-es-CR-Rosa
neural	Spanish, Cuba	es-CU	ai3-es-CU-Gabriel, ai3-es-CU-Rosario
neural	Spanish, Dominican Republic	es-DO	ai3-es-DO-Zoraida, ai3-es-do-Fernando
neural	Spanish, Ecuador	es-EC	ai3-es-EC-Jacob, ai3-es-EC-Cristina
neural	Spanish, Castilian (Spain)	es-ES	ai3-es-ES-Alvaro, ai3-es-ES-Elvira, ai3-es-ES-Lia, ai3-es-ES-Oscar, ai3-es-ES-Maura, ai3-es-ES-Juana, ai3-es-ES-Cruz, ai3-es-ES-Lorenzo, ai3-es-ES-Cristina, ai3-es-ES-Xiomara, ai3-es-ES-Domingo, ai3-es-ES-Silvio, ai3-es-ES-Carlos, ai3-es-ES-Viviana, ai3-es-ES-Ramiro, ai3-es-ES-Blanca, ai3-es-ES-MarianaV2

ai4-es-ES-Savannah, ai4-es-ES-Matlab

ai1-es-ES-Patricia, ai1-es-ES-Casper

ai2-es-ES-Vega, ai2-es-ES-Luciana, ai2-es-ES-Ricardo, ai2-es-ES-Ruben2, ai2-es-ES-Azura2, ai2-es-ES-Reyna2
neural	Spanish, Equatorial Guinea	es-GQ	ai3-es-GQ-Sebastian, ai3-es-GQ-Marcela
neural	Spanish, Guatemala	es-GT	ai3-es-GT-Leticia, ai3-es-GT-Ramiro
neural	Spanish, Honduras	es-HN	ai3-es-HN-Carlos, ai3-es-HN-Karla
neural	Spanish, Latin American	es-LA	ai4-es-LA-Luz
neural	Spanish, Mexican	es-MX	ai3-es-MX-Jorge, ai3-es-MX-Dalia, ai3-es-MX-Tadeo, ai3-es-MX-Lucia, ai3-es-MX-Ximena, ai3-es-MX-Emilio, ai3-es-MX-Romina, ai3-es-MX-Alexander, ai3-es-MX-Leonel, ai3-es-MX-Elisa, ai3-es-MX-Axel, ai3-es-MX-Elizabeth, ai3-es-MX-Fernanda, ai3-es-MX-Santiago, ai3-es-MX-Isabella

ai1-es-MX-Camila, ai1-es-MX-Luis
neural	Spanish, Nicaragua	es-NI	ai3-es-NI-Vidal, ai3-es-NI-Estrella
neural	Spanish, Panama	es-PA	ai3-es-PA-Domingo, ai3-es-PA-Belinda
neural	Spanish, Peru	es-PE	ai3-es-PE-Alex, ai3-es-PE-Camila
neural	Spanish, Puerto Rico	es-PR	ai3-es-PR-Karina, ai3-es-PR-Victor
neural	Spanish, Paraguay	es-PY	ai3-es-PY-Tomas, ai3-es-PY-Maria
neural	Spanish, El Salvador	es-SV	ai3-es-SV-Mateo, ai3-es-SV-Juana
neural	Spanish, US	es-US	ai4-es-US-Luz2

ai1-es-US-Lupe, ai1-es-US-Diego

ai3-es-US-Alberto, ai3-es-US-Paz

ai2-es-US-Manolito, ai2-es-US-Savanna, ai2-es-US-Orlando, ai2-es-US-Savanna2, ai2-es-US-Orlando2, ai2-es-US-Manolito2
neural	Spanish, Uruguay	es-UY	ai3-es-UY-Santino, ai3-es-UY-Valentina
neural	Spanish, Venezuela	es-VE	ai3-es-VE-Lucia, ai3-es-VE-Ricardo
neural	Estonian, Estonia	et-EE	ai3-et-EE-Tuudur, ai3-et-EE-Edenema
neural	Basque	eu-ES	ai3-eu-ES-Ximena, ai3-eu-ES-Leonel
neural	Persian (Iran)	fa-IR	ai3-fa-IR-Wadid, ai3-fa-IR-Naavya
neural	Finnish (Finland)	fi-FI	ai2-fi-FI-Karoliina

ai3-fi-FI-Noora, ai3-fi-FI-Harri, ai3-fi-FI-Selma

ai1-fi-FI-Marjatta
neural	Filipino (Philippines)	fil-PH	ai2-fil-PH-Nathan, ai2-fil-PH-Gabriel, ai2-fil-PH-Jennly, ai2-fil-PH-Camille

ai3-fil-PH-Gloria, ai3-fil-PH-Joshua
neural	French (Belgium)	fr-BE	ai3-fr-BE-Leonie, ai3-fr-BE-Gabriel

ai1-fr-BE-Elise
neural	French, Canadian	fr-CA	ai2-fr-CA-Paul, ai2-fr-CA-Scarlett, ai2-fr-CA-Christophe, ai2-fr-CA-MariePier

ai3-fr-CA-Jean, ai3-fr-CA-Sylvie, ai3-fr-CA-Kylian, ai3-fr-CA-BenoitV2

ai1-Gianna, ai1-fr-CA-Mylan

ai4-fr-CA-Avril
neural	French, Switzerland	fr-CH	ai3-fr-CH-Leandro, ai3-fr-CH-Lena
neural	French (France)	fr-FR	ai4-fr-FR-Blaise, ai4-fr-FR-Charles

ai2-fr-FR-Cassandra, ai2-fr-FR-Amandine, ai2-fr-FR-Erwan, ai2-fr-FR-Valentine, ai2-fr-FR-Dylan

ai3-fr-FR-Henri, ai3-fr-FR-Denise, ai3-fr-FR-Nevil, ai3-fr-FR-Claire, ai3-fr-FR-Roel, ai3-fr-FR-Tyssen, ai3-fr-FR-Liana, ai3-fr-FR-Austine, ai3-fr-FR-Cannan, ai3-fr-FR-Camille, ai3-fr-FR-Tayler, ai3-fr-FR-Manie, ai3-fr-FR-Emmy, ai3-fr-FR-Victoire, ai3-fr-FR-OdetteV2

ai1-fr-FR-Jeanne, ai1-fr-FR-Bernado
neural	Irish, Ireland	ga-IE	ai3-ga-IE-Rian, ai3-ga-IE-Eabha
neural	Galician (Spain)	gl-ES	ai3-gl-ES-Evita, ai3-gl-ES-Marcos
neural	Gujarati (India)	gu-IN	ai3-gu-IN-Prachi, ai3-gu-IN-Mihir

ai2-gu-IN-Varun, ai2-gu-IN-Minal
neural	Hebrew, Israel	he-IL	ai3-he-IL-Guy, ai3-he-IL-Shira
neural	Hindi (India)	hi-IN	ai2-hi-IN-Zoya, ai2-hi-IN-Anamika, ai2-hi-IN-Dhru, ai2-hi-IN-Nikhil

ai3-hi-IN-Madhur, ai3-hi-IN-Swara, ai3-hi-IN-AdityaV2, ai3-hi-IN-AanyaV2, ai3-hi-IN-Kavita, ai3-hi-IN-NikitaV2, ai3-hi-IN-SiddharthV2, ai3-hi-IN-GurdeepV2, ai3-hi-IN-DivyaV2, ai3-hi-IN-ChaitaliV2, ai3-hi-IN-Ankita, ai3-hi-IN-Karan

ai1-hi-IN-Kavya

ai13-hi-IN-Shreyas
neural	Croatian, Croatia	hr-HR	ai3-hr-HR-Vitomira, ai3-hr-HR-Dmitar
neural	Hungarian	hu-HU	ai3-hu-HU-Noemi, ai3-hu-HU-Tamas

ai2-hu-HU-Eszter
neural	Armenian (Armenia)	hy-AM	ai3-hy-AM-Tigran, ai3-hy-AM-Carine
neural	Indonesian	id-ID	ai2-id-ID-Putri, ai2-id-ID-David, ai2-id-ID-Henry, ai2-id-ID-Salsabilla

ai3-id-ID-Ardi, ai3-id-ID-Fitri
neural	Icelandic	is-IS	ai3-is-IS-Ulfr, ai3-is-IS-Svana
neural	Italian	it-IT	ai2-it-IT-Siliva, ai2-it-IT-Dario, ai2-it-IT-Federica, ai2-it-IT-Alessandro

ai3-it-IT-Diego, ai3-it-IT-Isabella, ai3-it-IT-Elsa, ai3-it-IT-Fabiola, ai3-it-IT-Valeria, ai3-it-IT-Regina, ai3-it-IT-Ludovica, ai3-it-IT-Aitana, ai3-it-IT-Matteo, ai3-it-IT-Natalia, ai3-it-IT-Tito, ai3-it-IT-Gerardo, ai3-it-IT-Ennio, ai3-it-IT-Massimo, ai3-it-IT-Francesco, ai3-it-IT-CaterinaV2

ai4-it-IT-Sara

ai1-it-IT-Viola, ai1-it-IT-Tommaso
neural	Japanese	ja-JP	ai2-ja-JP-Yuka, ai2-ja-JP-Ayaka, ai2-ja-JP-Masa, ai2-ja-JP-Taiyo

ai3-ja-JP-Nanami, ai3-ja-JP-Keita, ai3-ja-JP-Minato, ai3-ja-JP-Niko, ai3-ja-JP-Ren, ai3-ja-JP-Sakura, ai3-ja-JP-Himari

ai4-ja-JP-Akari

ai1-ja-JP-Haruto, ai1-ja-JP-Masako, ai1-ja-JP-Kanna
neural	Javanese (Indonesia)	jv-ID	ai3-jv-ID-Angkasa, ai3-jv-ID-Rimbo
neural	Georgian (Georgia)	ka-GE	ai3-ka-GE-Otar, ai3-ka-GE-Louisa
neural	Kazakh (Kazakhstan)	kk-KZ	ai3-kk-KZ-Kanat, ai3-kk-KZ-Batima
neural	Khmer (Cambodia)	km-KH	ai3-km-KH-Vanna, ai3-km-KH-Choum
neural	Kannada (India)	kn-IN	ai2-kn-IN-Aadi, ai2-kn-IN-Vaani

ai3-kn-IN-Vijay, ai3-kn-IN-Deepa
neural	Korean	ko-KR	ai2-ko-KR-Hannah, ai2-ko-KR-JiYeon, ai2-ko-KR-DongMin, ai2-ko-KR-Minseok

ai3-ko-KR-SunHi, ai3-ko-KR-InJoon, ai3-ko-KR-Bitna, ai3-ko-KR-Sena, ai3-ko-KR-Hyuk, ai3-ko-KR-Geon, ai3-ko-KR-Kyong, ai3-ko-KR-Yong, ai3-ko-KR-Yong, ai3-ko-KR-MyungV2

ai1-ko-KR-Seoyeon, ai1-ko-KR-Youngmi

ai4-ko-KR-Dalnim
neural	Lao (Laos)	lo-LA	ai3-lo-LA-Anuson, ai3-lo-LA-Sawan
neural	Lithuanian	lt-LT	ai3-lt-LT-Jokubas, ai3-lt-LT-Vasara
neural	Latvian, Latvia	lv-LV	ai3-lv-LV-Laura2, ai3-lv-LV-Edgar
neural	Macedonian (Republic of North Macedonia)	mk-MK	ai3-mk-MK-Risto, ai3-mk-MK-Eurydike
neural	Malayalam (India)	ml-IN	ai2-ml-IN-Harsh, ai2-ml-IN-Tina, ai2-ml-IN-Charu, ai2-ml-IN-Ashok

ai3-ml-IN-Indrajit, ai3-ml-IN-Revathi
neural	Mongolian (Mongolia)	mn-MN	ai3-mn-MN-Yagaan, ai3-mn-MN-Khasar
neural	Marathi (India)	mr-IN	ai3-mr-IN-Sandhya, ai3-mr-IN-Prashant

ai2-mr-IN-Komal, ai2-mr-IN-Rohan, ai2-mr-IN-Disha
neural	Malay, Malaysia	ms-MY	ai2-ms-MY-Marina, ai2-ms-MY-Aadam, ai2-ms-MY-Suzana, ai2-ms-MY-Zaafer

ai3-ms-MY-Osman, ai3-ms-MY-Yasmin
neural	Maltese, Malta	mt-MT	ai3-mt-MT-Alessia, ai3-mt-MT-Xavier
neural	Burmese (Myanmar)	my-MM	ai3-my-MM-Khine, ai3-my-MM-Inzali
neural	Norwegian	nb-NO	ai2-nb-NO-Margrete, ai2-nb-NO-Terese, ai2-nb-NO-Norah, ai2-nb-NO-Henrik, ai2-nb-NO-Lukas

ai3-nb-NO-Iselin, ai3-nb-NO-Magnus, ai3-nb-NO-Anita

ai1-nb-NO-Frida
neural	Nepali (Nepal)	ne-NP	ai3-ne-NP-Utsav, ai3-ne-NP-Chimini
neural	Dutch (Belgium)	nl-BE	ai3-nl-BE-Aldert, ai3-nl-BE-Marit

ai2-nl-BE-Capucine, ai2-nl-BE-Markus

ai1-nl-BE-Isa
neural	Dutch (Netherlands)	nl-NL	ai3-nl-NL-Colette, ai3-nl-NL-Maarten, ai3-nl-NL-Fenna

ai2-nl-NL-Arenda, ai2-nl-NL-Rogier, ai2-nl-NL-Roosje, ai2-nl-NL-Sterre, ai2-nl-NL-Gerben

ai1-nl-NL-Liva

ai4-nl-NL-Doutzen
neural	Oriya (India)	or-IN	ai3-or-IN-Bhoomika, ai3-or-IN-Shivendu
neural	Punjabi (India)	pa-IN	ai2-pa-IN-Daler, ai2-pa-IN-Chitra, ai2-pa-IN-Ranbir, ai2-pa-IN-Maahi

ai3-pa-IN-Anjali, ai3-pa-IN-Nihal
neural	Polish	pl-PL	ai2-pl-PL-Hanna, ai2-pl-PL-Julia, ai2-pl-PL-Wojciech, ai2-pl-PL-Franciszek, ai2-pl-PL-Alicja

ai3-pl-PL-Lena, ai3-pl-PL-Zofia, ai3-pl-PL-Kacper

ai1-pl-PL-Kalina
neural	Pashto (Afghanistan)	ps-AF	ai3-ps-AF-Shahpur, ai3-ps-AF-Naghma
neural	Portuguese, Brazilian	pt-BR	ai2-pt-BR-Keira, ai2-pt-BR-Paulo, ai2-pt-BR-Juliana

ai4-pt-BR-Fernanda

ai1-pt-BR-Camila, ai1-pt-BR-Bruno

ai3-pt-BR-Francisca, ai3-pt-BR-Antonio, ai3-pt-BR-Manuella, ai3-pt-BR-Alandra, ai3-pt-BR-Lucas, ai3-pt-BR-Humberto, ai3-pt-BR-Jaren, ai3-pt-BR-Rafael, ai3-pt-BR-Bernardo, ai3-pt-BR-Salvador, ai3-pt-BR-Leila, ai3-pt-BR-Yara, ai3-pt-BR-Rio, ai3-pt-BR-Alice, ai3-pt-BR-Giovanna, ai3-pt-BR-MatildeV2
neural	Portuguese	pt-PT	ai2-pt-PT-Margarida, ai2-pt-PT-Diogo, ai2-pt-PT-Ines, ai2-pt-PT-Gabriel

ai3-pt-PT-Fernanda, ai3-pt-PT-Raquel, ai3-pt-PT-Duarte

ai1-pt-PT-Laura
neural	Romanian	ro-RO	ai3-ro-RO-Alina, ai3-ro-RO-Alexandru

ai2-ro-RO-Corina
neural	Russian	ru-RU	ai2-ru-RU-Samara, ai2-ru-RU-Tianna, ai2-ru-RU-Czar, ai2-ru-RU-Igor, ai2-ru-RU-Tassa

ai3-ru-RU-Yelena, ai3-ru-RU-Dariya, ai3-ru-RU-Dmitry

ai5-ru-RU-Yuri, ai5-ru-RU-Vladimir, ai5-ru-RU-Alisa, ai5-ru-RU-Sofia, ai5-ru-RU-Konstantin, ai5-ru-RU-Dmitri, ai5-ru-RU-Ekaterina
neural	Sinhala (Sri Lanka)	si-LK	ai3-si-LK-Charuka, ai3-si-LK-Vedant
neural	Slovak (Slovakia)	sk-SK	ai2-sk-SK-Kristina

ai3-sk-SK-Viktoria, ai3-sk-SK-Lukas
neural	Slovenian (Slovenia)	sl-SI	ai3-sl-SI-Izabela, ai3-sl-SI-Patrik
neural	Somali (Somalia)	so-SO	ai3-so-SO-Fowsio, ai3-so-SO-Cumar
neural	Albanian (Albania)	sq-AL	ai3-sq-AL-Ilir, ai3-sq-AL-Anila
neural	Serbian, Cyrillic	sr-RS	ai3-sr-RS-Katarina, ai3-sr-RS-Nemanja
neural	Sundanese (Indonesia)	su-ID	ai3-su-ID-Pratam, ai3-su-ID-Cindy
neural	Swedish	sv-SE	ai3-sv-SE-Sofie, ai3-sv-SE-Mattias, ai3-sv-SE-Hillevi

ai2-sv-SE-Elsa, ai2-sv-SE-Emilie, ai2-sv-SE-Victor, ai2-sv-SE-Lea, ai2-sv-SE-Ludvig

ai1-sv-SE-Agnes
neural	Swahili (Kenya)	sw-KE	ai3-sw-KE-Obuya, ai3-sw-KE-Fanaka
neural	Swahili (Tanzania)	sw-TZ	ai3-sw-TZ-Peter, ai3-sw-TZ-Firyali
neural	Tamil (India)	ta-IN	ai3-ta-IN-Valluvar, ai3-ta-IN-Pallavi

ai2-ta-IN-Smita, ai2-ta-IN-Illayavan, ai2-ta-IN-Vihan, ai2-ta-IN-Bhanumathi
neural	Tamil (Sri Lanka)	ta-LK	ai3-ta-LK-Shreenika, ai3-ta-LK-Viraj
neural	Tamil (Malaysia)	ta-MY	ai3-ta-MY-Moshika, ai3-ta-MY-Surya
neural	Tamil (Singapore)	ta-SG	ai3-ta-SG-Jabin, ai3-ta-SG-Aaksara
neural	Telugu (India)	te-IN	ai3-te-IN-Mohan, ai3-te-IN-Shruti
neural	Thai (Thailand)	th-TH	ai3-th-TH-Narong, ai3-th-TH-Premwadee, ai3-th-TH-Achara

ai2-th-TH-Gamon
neural	Turkish	tr-TR	ai2-tr-TR-Neylan, ai2-tr-TR-Candana, ai2-tr-TR-Tabeeb, ai2-tr-TR-Roxelana, ai2-tr-TR-Gulizar

ai3-tr-TR-Emel

ai1-tr-TR-Burcu
neural	Ukrainian (Ukraine)	uk-UA	ai3-uk-UA-Olena, ai3-uk-UA-Pavlo

ai2-uk-UA-Aleksandra
neural	Urdu, India	ur-IN	ai3-ur-IN-Fatima, ai3-ur-IN-Salman
neural	Urdu, Pakistan	ur-PK	ai3-ur-PK-Aslam, ai3-ur-PK-Mehreen
neural	Uzbek (Uzbekistan)	uz-UZ	ai3-uz-UZ-Akbar, ai3-uz-UZ-Diliya
neural	Vietnamese (Vietnam)	vi-VN	ai2-vi-VN-Hyunh, ai2-vi-VN-Xuan, ai2-vi-VN-Thi, ai2-vi-VN-Binh

ai3-vi-VN-HoaiMy, ai3-vi-VN-Phuong
neural	Chinese (Wu, S)	wuu-CN	ai3-wuu-CN-Jiang, ai3-wuu-CN-SunLi
neural	Chinese (Cantonese, S)	yue-CN	ai3-yue-CN-Wang, ai3-yue-CN-Yang
neural	Chinese, Cantonese	zh-HK	ai3-zh-HK-WanLung, ai3-zh-HK-HiuGaai, ai3-zh-HK-HiuMaan
neural	Zulu (South Africa)	zu-ZA	ai3-zu-ZA-Gauteng, ai3-zu-ZA-Nonhle
""";

voices = []

# Process the text data
# We iterate line by line.
# If a line starts with "neural", it's a primary definition.
# If it doesn't, but contains voice IDs (ai...), it's a continuation of the previous "neural" entry (additional voices).

current_entry = None
lines = text_data.strip().split('\n')

for line in lines:
    line = line.strip()
    if not line:
        continue
    
    parts = re.split(r'\t+', line)
    
    if parts[0] == 'neural' and len(parts) >= 4:
        # New entry
        engine = parts[0]
        language = parts[1]
        language_code = parts[2]
        # voice_ids part can be comma separated
        voice_ids_str = parts[3]
        
        # Split voice IDs and clean them
        voice_ids = [v.strip() for v in voice_ids_str.split(',') if v.strip()]
        
        current_entry = {
            "Language": language,
            "LanguageCode": language_code,
            "Voices": voice_ids
        }
        voices.append(current_entry)
        
    elif current_entry and (line.startswith('ai') or ',' in line):
        # Continuation of voices for the current entry
        additional_voices = [v.strip() for v in line.split(',') if v.strip()]
        current_entry["Voices"].extend(additional_voices)

# Sort by language name
voices.sort(key=lambda x: x["Language"])

output_file = "config/voicemaker_voices.json"
with open(output_file, "w", encoding="utf-8") as f:
    json.dump(voices, f, indent=4, ensure_ascii=False)

print(f"Successfully generated {output_file} with {len(voices)} languages.")
