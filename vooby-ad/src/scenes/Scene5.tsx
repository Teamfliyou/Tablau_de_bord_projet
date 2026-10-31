import React from 'react';
import {Photo} from '../components/Photo';
import {FrenchFlag, GoldRule, Reveal, Title} from '../components/Text';
import {SceneShell, Shade, SmallLogo, Vignette} from '../components/Scene';
import {BEAT} from '../constants';

export const Scene5: React.FC<{duration: number}> = ({duration}) => (
	<SceneShell>
		<Photo src="photos/01-face-studio.jpg" from={{scale: 1.4, x: 0, y: 2}} to={{scale: 1.04, x: 0, y: 0}} duration={duration} origin="50% 55%" />
		{/* voile sombre : la photo studio est très claire, on assure la lisibilité du texte blanc */}
		<div style={{position: 'absolute', inset: 0, background: 'rgba(10,10,11,0.28)'}} />
		<Shade top={0.45} bottom={0.78} flat={0} />
		<Vignette />
		<SmallLogo at={2} />
		<div style={{position: 'absolute', left: 70, right: 70, top: 1190, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 20}}>
			<GoldRule at={Math.round(BEAT * 0.5)} width={120} />
			<Reveal at={Math.round(BEAT * 1)}>
				<Title size={78}>Fabriqué dans notre atelier</Title>
			</Reveal>
			<Reveal at={Math.round(BEAT * 2)}>
				<div style={{display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 26}}>
					<Title size={78}>à Saint-Omer</Title>
					<FrenchFlag height={58} />
				</div>
			</Reveal>
		</div>
	</SceneShell>
);
