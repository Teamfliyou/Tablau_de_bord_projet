import React from 'react';
import {AbsoluteFill, Composition, Series} from 'remotion';
import {Footer} from './components/Common';
import {Avancement, AVANCEMENT_DURATION} from './scenes/Avancement';
import {Permis, PERMIS_DURATION} from './scenes/Permis';
import {Telephone, TELEPHONE_DURATION} from './scenes/Telephone';
import {C, FPS} from './theme';

/**
 * Vidéo pour l'écran TV : les 3 visuels s'enchaînent, chacun avec ses
 * animations d'entrée et de sortie. La vidéo commence et finit sur un fond
 * vide (bandeau du bas fixe), elle peut donc tourner en boucle sans coupure.
 */
const MosqueeTV: React.FC = () => (
	<AbsoluteFill style={{background: C.bg}}>
		<Series>
			<Series.Sequence durationInFrames={AVANCEMENT_DURATION}>
				<Avancement />
			</Series.Sequence>
			<Series.Sequence durationInFrames={PERMIS_DURATION}>
				<Permis />
			</Series.Sequence>
			<Series.Sequence durationInFrames={TELEPHONE_DURATION}>
				<Telephone />
			</Series.Sequence>
		</Series>
		<Footer />
	</AbsoluteFill>
);

export const RemotionRoot: React.FC = () => (
	<>
		<Composition
			id="MosqueeTV"
			component={MosqueeTV}
			durationInFrames={AVANCEMENT_DURATION + PERMIS_DURATION + TELEPHONE_DURATION}
			fps={FPS}
			width={1920}
			height={1080}
		/>
		<Composition id="Avancement" component={AvancementScene} durationInFrames={AVANCEMENT_DURATION} fps={FPS} width={1920} height={1080} />
		<Composition id="Permis" component={PermisScene} durationInFrames={PERMIS_DURATION} fps={FPS} width={1920} height={1080} />
		<Composition id="Telephone" component={TelephoneScene} durationInFrames={TELEPHONE_DURATION} fps={FPS} width={1920} height={1080} />
	</>
);

function Scene(Comp: React.FC): React.FC {
	const Wrapped: React.FC = () => (
		<AbsoluteFill style={{background: C.bg}}>
			<Comp />
			<Footer />
		</AbsoluteFill>
	);
	return Wrapped;
}

const AvancementScene = Scene(Avancement);
const PermisScene = Scene(Permis);
const TelephoneScene = Scene(Telephone);
