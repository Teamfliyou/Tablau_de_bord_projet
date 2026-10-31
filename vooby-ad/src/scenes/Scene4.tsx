import React from 'react';
import {Photo} from '../components/Photo';
import {GoldRule, Reveal, Title} from '../components/Text';
import {SceneShell, Shade, SmallLogo, Vignette} from '../components/Scene';
import {BEAT} from '../constants';

export const Scene4: React.FC<{duration: number}> = ({duration}) => (
	<SceneShell>
		<Photo src="photos/05-coffre.jpg" from={{scale: 1.0, x: 0, y: 0}} to={{scale: 1.5, x: -3, y: 1}} duration={duration} origin="52% 46%" />
		<Shade top={0.4} bottom={0.72} flat={0} />
		<Vignette />
		<SmallLogo at={2} />
		<div style={{position: 'absolute', left: 70, right: 70, top: 1230, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 20}}>
			<GoldRule at={Math.round(BEAT * 0.5)} width={120} />
			<Reveal at={Math.round(BEAT * 1)}>
				<Title size={100}>Un coffre intégré</Title>
			</Reveal>
			<Reveal at={Math.round(BEAT * 2)}>
				<Title size={100}>pour tout ranger</Title>
			</Reveal>
		</div>
	</SceneShell>
);
